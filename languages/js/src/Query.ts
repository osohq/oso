import type { Query as FfiQuery } from './polar_wasm_api';

import { createInterface } from 'readline';

import { parseQueryEvent } from './helpers';
import { DuplicateInstanceRegistrationError, InvalidCallError } from './errors';
import { Host } from './Host';
import type {
  Debug,
  ExternalCall,
  ExternalIsa,
  ExternalIsSubspecializer,
  ExternalUnify,
  MakeExternal,
  PolarTerm,
  QueryEvent,
  QueryResult,
  Result,
} from './types';
import { processMessage } from './messages';
import { isAsyncIterator, isIterableIterator, QueryEventKind } from './types';

export class Query {
  #ffiQuery: FfiQuery;
  #calls: Map<number, AsyncGenerator>;
  #host: Host;
  results: QueryResult;

  constructor(ffiQuery: FfiQuery, host: Host) {
    this.#ffiQuery = ffiQuery;
    this.#calls = new Map();
    this.#host = host;
    this.results = this.start();
  }

  private processMessages() {
    while (true) {
      let msg = this.#ffiQuery.nextMessage();
      if (msg === undefined) break;
      processMessage(msg);
    }
  }

  private questionResult(result: boolean, callId: number): void {
    this.#ffiQuery.questionResult(callId, result);
  }

  private async registerCall(
    field: string,
    callId: number,
    instance: PolarTerm,
    args?: PolarTerm[]
  ): Promise<void> {
    if (this.#calls.has(callId)) return;
    const receiver = await this.#host.toJs(instance);
    let value = receiver[field];
    if (args !== undefined) {
      if (typeof value === 'function') {
        // If value is a function, call it with the provided args.
        const jsArgs = args!.map(async a => await this.#host.toJs(a));
        value = receiver[field](...(await Promise.all(jsArgs)));
      } else {
        // Error on attempt to call non-function.
        throw new InvalidCallError(receiver, field);
      }
    }
    const generator = (async function* () {
      if (isIterableIterator(value)) {
        // If the call result is an iterable iterator, yield from it.
        yield* value;
      } else if (isAsyncIterator(value)) {
        // Same for async iterators.
        for await (const result of value) {
          yield result;
        }
      } else {
        // Otherwise, yield it.
        yield value;
      }
    })();
    this.#calls.set(callId, generator);
  }

  private callResult(callId: number, result?: string): void {
    this.#ffiQuery.callResult(callId, result);
  }

  private async nextCallResult(callId: number): Promise<string | undefined> {
    const { done, value } = await this.#calls.get(callId)!.next();
    if (done) return undefined;
    return JSON.stringify(this.#host.toPolar(value));
  }

  private applicationError(message: string): void {
    this.#ffiQuery.appError(message);
  }

  private async handleCall(
    attr: string,
    callId: number,
    instance: PolarTerm,
    args?: PolarTerm[]
  ): Promise<void> {
    let result;
    try {
      await this.registerCall(attr, callId, instance, args);
      result = await this.nextCallResult(callId);
    } catch (e) {
      if (e instanceof TypeError || e instanceof InvalidCallError) {
        this.applicationError(e.message);
      } else {
        throw e;
      }
    } finally {
      this.callResult(callId, result);
    }
  }

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
            const {
              attribute,
              callId,
              instance,
              args,
            } = event.data as ExternalCall;
            await this.handleCall(attribute, callId, instance, args);
            break;
          }
          case QueryEventKind.ExternalIsSubspecializer: {
            const {
              instanceId,
              leftTag,
              rightTag,
              callId,
            } = event.data as ExternalIsSubspecializer;
            const answer = await this.#host.isSubspecializer(
              instanceId,
              leftTag,
              rightTag
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
          case QueryEventKind.ExternalUnify: {
            const { leftId, rightId, callId } = event.data as ExternalUnify;
            const answer = await this.#host.unify(leftId, rightId);
            this.questionResult(answer, callId);
            break;
          }
          case QueryEventKind.Debug:
            const { message } = event.data as Debug;
            if (message) console.log(message);
            createInterface({
              input: process.stdin,
              output: process.stdout,
              prompt: 'debug> ',
              tabSize: 4,
            }).on('line', line => {
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
