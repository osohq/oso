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
import { Relation } from './dataFiltering';
import type { FilterKind } from './dataFiltering';

function getLogLevelsFromEnv() {
  if (typeof process?.env === 'undefined') return [undefined, undefined];
  return [process.env.RUST_LOG, process.env.POLAR_LOG];
}

/**
 * A single Polar query.
 *
 * @internal
 */
export class Query {
  #ffiQuery: FfiQuery;
  #calls: Map<number, AsyncGenerator>;
  #host: Host;
  results: QueryResult;

  constructor(ffiQuery: FfiQuery, host: Host, bindings?: Map<string, unknown>) {
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
    this.#ffiQuery.bind(name, JSON.stringify(this.#host.toPolar(value)));
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
  private callResult(callId: number, result?: string): void {
    this.#ffiQuery.callResult(callId, result);
  }

  /**
   * Retrieve the next result from a registered call and prepare it for
   * transmission back to the Polar VM.
   *
   * @internal
   */
  private async nextCallResult(callId: number): Promise<string | undefined> {
    const call = this.#calls.get(callId);
    if (call === undefined) throw new Error('invalid call');
    const { done, value } = await call.next(); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
    if (done) return undefined;
    return JSON.stringify(this.#host.toPolar(value));
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
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private async handleRelation(receiver: any, rel: Relation): Promise<unknown> {
    // TODO(gj|gw): we should add validation for UserType relations once we
    // have a nice hook where we know every class has been registered
    // (e.g., once we enforce that all registerCalls() have to happen
    // before loadFiles()).
    const typ = this.#host.getType(rel.otherType);
    if (typ === undefined) throw new UnregisteredClassError(rel.otherType);

    // NOTE(gj): disabling ESLint for following line b/c we're fine if
    // `receiver[rel.myField]` blows up -- we catch the error and relay it to
    // the core in `handleCall`.
    const value = receiver[rel.myField] as unknown; // eslint-disable-line

    // Use the fetcher for the other type to traverse
    // the relationship.
    const filter = { kind: 'Eq' as FilterKind, value, field: rel.otherField };
    const query = await typ.buildQuery([filter]); // eslint-disable-line @typescript-eslint/no-unsafe-assignment
    const results = await typ.execQuery(query);
    if (rel.kind === 'one') {
      if (results.length !== 1)
        throw new Error(`Wrong number of parents: ${results.length}`);
      return results[0]; // eslint-disable-line @typescript-eslint/no-unsafe-return
    } else {
      return results; // eslint-disable-line @typescript-eslint/no-unsafe-return
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
      const rel = this.#host.getType(receiver?.constructor)?.fields?.get(attr);
      if (rel instanceof Relation) {
        value = await this.handleRelation(receiver, rel);
      } else {
        // NOTE(gj): disabling ESLint for following line b/c we're fine if
        // `receiver[attr]` blows up -- we catch the error and relay it to the
        // core below.
        value = (receiver as any)[attr]; // eslint-disable-line
        if (args !== undefined) {
          if (typeof value === 'function') {
            // If value is a function, call it with the provided args.
            const jsArgs = await Promise.all(
              args.map(async a => await this.#host.toJs(a))
            );
            // NOTE(gj): disabling ESLint for following line b/c we know
            // `receiver[attr]` (A) won't blow up (because if it was going to
            // it already would've happened above) and (B) is a function
            // (thanks to the `typeof value === 'function'` check above).
            //
            // The function invocation could still blow up with a `TypeError`
            // if `receiver[attr]` is a class constructor (e.g., if instance
            // were something like `{x: class{}}`), but that'll be caught &
            // relayed to the core down below.
            value = ((receiver as any)[attr] as CallableFunction)(...jsArgs); // eslint-disable-line
          } else {
            // Error on attempt to call non-function.
            throw new InvalidCallError(receiver, attr);
          }
        } else {
          // If value isn't a property anywhere in receiver's prototype chain,
          // throw an error.
          //
          // NOTE(gj): disabling TS for following line b/c we're fine if `attr
          // in receiver` blows up -- we catch the error and relay it to the
          // core below.
          //
          // eslint-disable-next-line @typescript-eslint/ban-ts-comment
          // @ts-ignore
          if (value === undefined && !(attr in receiver)) {
            throw new InvalidAttributeError(receiver, attr);
          }
        }
      }
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
      value = JSON.stringify(this.#host.toPolar(value));
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
