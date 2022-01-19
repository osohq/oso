import { createInterface } from 'readline';

import type { Query as FfiQuery } from './polar_wasm_api';

import { parseQueryEvent } from './helpers';
import {
  DuplicateInstanceRegistrationError,
  InvalidAttributeError,
  InvalidCallError,
  InvalidIteratorError,
  UnregisteredClassError,
} from './errors';
import { Host } from './Host';
import type {
  Debug,
  ExternalCall,
  ExternalIsa,
  ExternalIsaWithPath,
  ExternalIsSubclass,
  ExternalIsSubspecializer,
  ExternalOp,
  MakeExternal,
  NextExternal,
  obj,
  NullishOrHasConstructor,
  PolarTerm,
  QueryResult,
  Result,
} from './types';
import type { Message } from './messages';
import { processMessage } from './messages';
import {
  isAsyncIterable,
  isIterable,
  isIterableIterator,
  QueryEventKind,
} from './types';
import { Relation, Filter, FilterCondition } from './filter';

function getLogLevelsFromEnv() {
  if (typeof process?.env === 'undefined') return [undefined, undefined];
  return [process.env.RUST_LOG, process.env.POLAR_LOG];
}

/**
 * A single Polar query.
 *
 * @internal
 */
export class Query<AdapterQuery, Resource> {
  #ffiQuery: FfiQuery;
  #calls: Map<number, AsyncGenerator>;
  #host: Host<AdapterQuery, Resource>;
  results: QueryResult;

  constructor(
    ffiQuery: FfiQuery,
    host: Host<AdapterQuery, Resource>,
    bindings?: Map<string, unknown>
  ) {
    ffiQuery.setLoggingOptions(...getLogLevelsFromEnv());
    this.#ffiQuery = ffiQuery;
    this.#calls = new Map();
    this.#host = host;

    if (bindings) for (const [k, v] of bindings) this.bind(k, v);

    this.results = this.start();
  }

  /**
   * Process messages received from the Polar VM.
   *
   * @internal
   */
  private bind(name: string, value: unknown) {
    this.#ffiQuery.bind(name, this.#host.toPolar(value));
  }

  /**
   * Process messages received from the Polar VM.
   *
   * @internal
   */
  private processMessages() {
    for (;;) {
      const msg = this.#ffiQuery.nextMessage() as Message | undefined;
      if (msg === undefined) break;
      processMessage(msg);
    }
  }

  /**
   * Send result of predicate check back to the Polar VM.
   *
   * @internal
   */
  private questionResult(result: boolean, callId: number): void {
    this.#ffiQuery.questionResult(callId, result);
  }

  /**
   * Send next result of JavaScript method call or property lookup to the Polar
   * VM.
   *
   * @internal
   */
  private callResult(callId: number, result?: PolarTerm): void {
    this.#ffiQuery.callResult(callId, result);
  }

  /**
   * Retrieve the next result from a registered call and prepare it for
   * transmission back to the Polar VM.
   *
   * @internal
   */
  private async nextCallResult(callId: number): Promise<PolarTerm | undefined> {
    const call = this.#calls.get(callId);
    if (call === undefined) throw new Error('invalid call');
    const { done, value } = await call.next(); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
    if (done) return undefined;
    return this.#host.toPolar(value);
  }

  /**
   * Send application error back to the Polar VM.
   *
   * @internal
   */
  private applicationError(message: string): void {
    this.#ffiQuery.appError(message);
  }

  /**
   * Handle an external call on a relation.
   *
   * @internal
   */
  private async handleRelation(
    receiver: obj,
    rel: Relation
  ): Promise<Resource | Resource[]> {
    // TODO(gj|gw): we should add validation for UserType relations once we
    // have a nice hook where we know every class has been registered
    // (e.g., once we enforce that all registerCalls() have to happen
    // before loadFiles()).
    const typ = this.#host.getType(rel.otherType);
    if (typ === undefined) throw new UnregisteredClassError(rel.otherType);

    const value = receiver[rel.myField];
    const condition: FilterCondition = {
      lhs: {
        typeName: rel.otherType,
        fieldName: rel.otherField,
      },
      cmp: 'Eq',
      rhs: { value },
    };
    const filter: Filter = {
      model: rel.otherType,
      relations: [],
      conditions: [[condition]],
      types: this.#host.types,
    };
    const query = this.#host.adapter.buildQuery(filter);
    const results = await this.#host.adapter.executeQuery(query);

    if (rel.kind === 'one') {
      if (results.length !== 1)
        throw new Error(`Wrong number of parents: ${results.length}`);
      return results[0];
    } else {
      return results;
    }
  }

  /**
   * Handle an application call.
   *
   * @internal
   */
  private async handleCall(
    attr: string,
    callId: number,
    instance: PolarTerm,
    args?: PolarTerm[]
  ): Promise<void> {
    let value;
    try {
      const receiver = (await this.#host.toJs(
        instance
      )) as NullishOrHasConstructor;

      if (receiver === null || receiver === undefined)
        throw new (args ? InvalidCallError : InvalidAttributeError)(
          receiver,
          attr
        );

      const rel = this.#host.getType(receiver.constructor)?.fields?.get(attr);
      const self = receiver as obj;

      if (rel instanceof Relation) value = await this.handleRelation(self, rel);
      else if (args) {
        if (typeof self[attr] !== 'function')
          throw new InvalidCallError(receiver, attr);
        const jsArgs = await Promise.all(args.map(a => this.#host.toJs(a)));
        value = (self[attr] as CallableFunction)(...jsArgs); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
      } else if (!(attr in self))
        throw new InvalidAttributeError(receiver, attr);
      else value = self[attr];
    } catch (e) {
      if (
        e instanceof TypeError ||
        e instanceof InvalidCallError ||
        e instanceof InvalidAttributeError
      ) {
        this.applicationError(e.message);
      } else {
        throw e;
      }
    } finally {
      // resolve promise if necessary
      // convert result to JSON and return
      value = await Promise.resolve(value); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
      value = this.#host.toPolar(value);
      this.callResult(callId, value);
    }
  }

  private async handleNextExternal(callId: number, iterable: PolarTerm) {
    if (!this.#calls.has(callId)) {
      const value = await this.#host.toJs(iterable);
      const generator = (async function* () {
        if (isIterableIterator(value)) {
          // If the call result is an iterable iterator, yield from it.
          yield* value;
        } else if (isAsyncIterable(value)) {
          // Same for async iterators.
          for await (const result of value) {
            yield result;
          }
        } else if (isIterable(value)) {
          for (const result of value) {
            yield result;
          }
        } else {
          // Otherwise, error
          throw new InvalidIteratorError(typeof value);
        }
      })();
      this.#calls.set(callId, generator);
    }
    const result = await this.nextCallResult(callId);
    this.callResult(callId, result);
  }

  /**
   * Create an `AsyncGenerator` that can be polled to advance the query loop.
   *
   * @internal
   */
  private async *start(): QueryResult {
    try {
      while (true) {
        const nextEvent = this.#ffiQuery.nextEvent(); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
        this.processMessages();
        const event = parseQueryEvent(nextEvent);
        switch (event.kind) {
          case QueryEventKind.Done:
            return;
          case QueryEventKind.Result: {
            const { bindings } = event.data as Result;
            const transformed: Map<string, unknown> = new Map();
            for (const [k, v] of bindings) {
              transformed.set(k, await this.#host.toJs(v));
            }
            yield transformed;
            break;
          }
          case QueryEventKind.MakeExternal: {
            const { instanceId, tag, fields } = event.data as MakeExternal;
            if (this.#host.hasInstance(instanceId))
              throw new DuplicateInstanceRegistrationError(instanceId);
            await this.#host.makeInstance(tag, fields, instanceId);
            break;
          }
          case QueryEventKind.ExternalCall: {
            const { attribute, callId, instance, args } =
              event.data as ExternalCall;
            await this.handleCall(attribute, callId, instance, args);
            break;
          }
          case QueryEventKind.ExternalIsSubspecializer: {
            const { instanceId, leftTag, rightTag, callId } =
              event.data as ExternalIsSubspecializer;
            const answer = await this.#host.isSubspecializer(
              instanceId,
              leftTag,
              rightTag
            );
            this.questionResult(answer, callId);
            break;
          }
          case QueryEventKind.ExternalIsSubclass: {
            const { leftTag, rightTag, callId } =
              event.data as ExternalIsSubclass;
            const answer = this.#host.isSubclass(leftTag, rightTag);
            this.questionResult(answer, callId);
            break;
          }
          case QueryEventKind.ExternalOp: {
            const { args, callId, operator } = event.data as ExternalOp;
            const answer = await this.#host.externalOp(
              operator,
              args[0],
              args[1]
            );
            this.questionResult(answer, callId);
            break;
          }
          case QueryEventKind.ExternalIsa: {
            const { instance, tag, callId } = event.data as ExternalIsa;
            const answer = await this.#host.isa(instance, tag);
            this.questionResult(answer, callId);
            break;
          }
          case QueryEventKind.ExternalIsaWithPath: {
            const { baseTag, path, classTag, callId } =
              event.data as ExternalIsaWithPath;
            const answer = await this.#host.isaWithPath(
              baseTag,
              path,
              classTag
            );
            this.questionResult(answer, callId);
            break;
          }
          case QueryEventKind.NextExternal: {
            const { callId, iterable } = event.data as NextExternal;
            await this.handleNextExternal(callId, iterable);
            break;
          }
          case QueryEventKind.Debug: {
            if (typeof createInterface !== 'function') {
              console.warn('debug events not supported in browser Oso');
              break;
            }
            const { message } = event.data as Debug;
            if (message) console.log(message);
            createInterface({
              input: process.stdin,
              output: process.stdout,
              prompt: 'debug> ',
              tabSize: 4,
            }).on('line', (line: string) => {
              const trimmed = line.trim().replace(/;+$/, '');
              const command = this.#host.toPolar(trimmed);
              this.#ffiQuery.debugCommand(JSON.stringify(command));
              this.processMessages();
            });
            break;
          }
          default: {
            const _: never = event.kind;
            return _;
          }
        }
      }
    } finally {
      this.#ffiQuery.free();
    }
  }
}
