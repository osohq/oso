import type { Query as FfiQuery } from './polar_wasm_api';

const createInterface = require('readline')?.createInterface;

import { parseQueryEvent } from './helpers';
import {
  DuplicateInstanceRegistrationError,
  InvalidCallError,
  InvalidIteratorError,
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
  PolarTerm,
  QueryEvent,
  QueryResult,
  Result,
} from './types';
import { processMessage } from './messages';
import { isAsyncIterator, isIterableIterator, QueryEventKind } from './types';
import { Constraint, Relation } from './dataFiltering';

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

  constructor(ffiQuery: FfiQuery, host: Host, bindings?: Map<string, any>) {
    ffiQuery.setLoggingOptions(...getLogLevelsFromEnv());
    this.#ffiQuery = ffiQuery;
    this.#calls = new Map();
    this.#host = host;

    for (const k in bindings) {
      this.bind(k, bindings.get(k));
    }

    this.results = this.start();
  }

  /**
   * Process messages received from the Polar VM.
   *
   * @internal
   */
  private bind(name: string, value: any) {
    this.#ffiQuery.bind(name, JSON.stringify(this.#host.toPolar(value)));
  }

  /**
   * Process messages received from the Polar VM.
   *
   * @internal
   */
  private processMessages() {
    while (true) {
      let msg = this.#ffiQuery.nextMessage();
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
    const { done, value } = await this.#calls.get(callId)!.next();
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
      const receiver = await this.#host.toJs(instance);
      const userTypes = this.#host.types;

      // Check if it's a relationship
      if (receiver != undefined) {
        const userType = userTypes.get(receiver.constructor);
        const clsName = userType?.name;
        const typedef = userType?.fields;
        const fieldType = typedef?.get(attr);
        if (fieldType != null) {
          if (fieldType instanceof Relation) {
            const typ = userTypes.get(fieldType.otherType)!;
            // Use the fetcher for the other type to traverse
            // the relationship.
            const constraint = new Constraint(
              'Eq',
              fieldType.otherField,
              receiver[fieldType.myField]
            );
            let query = await Promise.resolve(typ.buildQuery!([constraint]));
            let results = await Promise.resolve(typ.execQuery!(query));
            if (fieldType.kind == 'one') {
              if (results.length != 1)
                throw new Error('Wrong number of parents: ' + results.length);
              value = results[0];
            } else {
              value = results;
            }
          }
        }
      }
      if (value == undefined) {
        value = receiver[attr];
        if (args !== undefined) {
          if (typeof value === 'function') {
            // If value is a function, call it with the provided args.
            const jsArgs = args!.map(async a => await this.#host.toJs(a));
            value = receiver[attr](...(await Promise.all(jsArgs)));
          } else {
            // Error on attempt to call non-function.
            throw new InvalidCallError(receiver, attr);
          }
        }
      }
    } catch (e) {
      if (e instanceof TypeError || e instanceof InvalidCallError) {
        this.applicationError(e.message);
      } else {
        throw e;
      }
    } finally {
      // resolve promise if necessary
      // convert result to JSON and return
      value = await Promise.resolve(value);
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
        } else if (isAsyncIterator(value)) {
          // Same for async iterators.
          for await (const result of value) {
            yield result;
          }
        } else if (Symbol.iterator in Object(value)) {
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
        const nextEvent = this.#ffiQuery.nextEvent();
        this.processMessages();
        const event: QueryEvent = parseQueryEvent(nextEvent);
        switch (event.kind) {
          case QueryEventKind.Done:
            return;
          case QueryEventKind.Result:
            const { bindings } = event.data as Result;
            const transformed: Map<string, any> = new Map();
            for (const [k, v] of bindings.entries()) {
              transformed.set(k, await this.#host.toJs(v));
            }
            yield transformed;
            break;
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
            const answer = await this.#host.isSubclass(leftTag, rightTag);
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
          case QueryEventKind.Debug:
            if (createInterface == null) {
              console.warn('debug events not supported in browser oso');
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
      }
    } finally {
      this.#ffiQuery.free();
    }
  }
}
