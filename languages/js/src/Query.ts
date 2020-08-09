import type { Query as FfiQuery } from '../dist/polar_wasm_api';

import { createInterface } from 'readline';

import { parseQueryEvent } from './helpers';
import { DuplicateInstanceRegistrationError, InvalidCallError } from './errors';
import { Host } from './Host';
import type {
  PolarValue,
  QueryEvent,
  QueryResult,
  Result,
  MakeExternal,
  ExternalCall,
  ExternalIsa,
  ExternalIsSubspecializer,
  ExternalUnify,
  Debug,
} from './types';
import { isGenerator, isGeneratorFunction, QueryEventKind } from './types';

export class Query {
  #ffiQuery: FfiQuery;
  #calls: Map<bigint, Generator>;
  #host: Host;
  results: QueryResult;

  constructor(ffiQuery: FfiQuery, host: Host) {
    this.#ffiQuery = ffiQuery;
    this.#calls = new Map();
    this.#host = host;
    this.results = this.start();
  }

  private questionResult(result: boolean, callId: bigint): void {
    this.#ffiQuery.questionResult(callId, result);
  }

  private registerCall(
    attr: string,
    callId: bigint,
    instance: PolarValue,
    args: PolarValue[]
  ): void {
    if (this.#calls.has(callId)) return;
    const jsArgs = args.map(a => this.#host.toJs(a));
    const jsInstance = this.#host.toJs(instance);
    const jsAttr = Reflect.get(jsInstance, attr);
    if (jsAttr === undefined) throw new InvalidCallError(attr, jsInstance);
    let result: Generator;
    if (isGenerator(jsAttr)) {
      let call;
      switch (jsArgs.length) {
        case 0:
          call = jsAttr.next();
          break;
        case 1:
          call = jsAttr.next(jsArgs[0]);
          break;
        default:
          // The Generator#next method only takes 0 or 1 args.
          throw new InvalidCallError(attr, jsInstance);
      }
      // TODO(gj): is this correct?
      result = (function* () {
        while (true) {
          const { done, value } = call;
          if (done) return;
          yield value;
        }
      })();
    } else if (isGeneratorFunction(jsAttr)) {
      // TODO(gj): is this correct?
      result = jsAttr(...jsArgs);
    } else if (typeof jsAttr === 'function') {
      result = (function* () {
        yield jsAttr(...jsArgs);
      })();
    } else {
      // Blow up if jsArgs is not [] since the user is attempting to invoke +
      // pass args to something that isn't callable.
      if (jsArgs.length > 0) throw new InvalidCallError(attr, jsInstance);
      result = (function* () {
        yield jsAttr;
      })();
    }
    this.#calls.set(callId, result);
  }

  private callResult(callId: bigint, result?: string): void {
    this.#ffiQuery.callResult(callId, result);
  }

  private nextCallResult(callId: bigint): string | undefined {
    const { done, value } = this.#calls.get(callId)!.next();
    // TODO(gj): should this only check 'done'?
    if (done || value === null || value === undefined) return undefined;
    return JSON.stringify(this.#host.toPolarTerm(value));
  }

  private applicationError(message: string): void {
    this.#ffiQuery.appError(message);
  }

  private handleCall(
    attr: string,
    callId: bigint,
    instance: PolarValue,
    args: PolarValue[]
  ): void {
    let result;
    try {
      this.registerCall(attr, callId, instance, args);
      result = this.nextCallResult(callId);
    } catch (e) {
      if (e instanceof InvalidCallError) {
        this.applicationError(e.message);
      } else {
        throw e;
      }
    } finally {
      this.callResult(callId, result);
    }
  }

  private *start(): QueryResult {
    while (true) {
      const nextEvent = this.#ffiQuery.nextEvent();
      const event: QueryEvent = parseQueryEvent(nextEvent);
      switch (event.kind) {
        case QueryEventKind.Done:
          return null;
        case QueryEventKind.Result:
          const { bindings } = event.data as Result;
          const transformed: Map<string, any> = new Map();
          for (const [k, v] of bindings.entries()) {
            transformed.set(k, this.#host.toJs(v));
          }
          yield transformed;
          break;
        case QueryEventKind.MakeExternal: {
          const { instanceId, tag, fields } = event.data as MakeExternal;
          if (this.#host.hasInstance(instanceId))
            throw new DuplicateInstanceRegistrationError(instanceId);
          this.#host.makeInstance(tag, fields, instanceId);
          break;
        }
        case QueryEventKind.ExternalCall: {
          const {
            attribute,
            callId,
            instance,
            args,
          } = event.data as ExternalCall;
          this.handleCall(attribute, callId, instance, args);
          break;
        }
        case QueryEventKind.ExternalIsSubspecializer: {
          const {
            instanceId,
            leftTag,
            rightTag,
            callId,
          } = event.data as ExternalIsSubspecializer;
          const answer = this.#host.isSubspecializer(
            instanceId,
            leftTag,
            rightTag
          );
          this.questionResult(answer, callId);
          break;
        }
        case QueryEventKind.ExternalIsa: {
          const { instanceId, tag, callId } = event.data as ExternalIsa;
          const answer = this.#host.isa(instanceId, tag);
          this.questionResult(answer, callId);
          break;
        }
        case QueryEventKind.ExternalUnify: {
          const { leftId, rightId, callId } = event.data as ExternalUnify;
          const answer = this.#host.unify(leftId, rightId);
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
            const command = this.#host.toPolarTerm(trimmed);
            this.#ffiQuery.debugCommand(JSON.stringify(command));
          });
      }
    }
  }
}
