import { createInterface } from 'readline';

import { Query as FfiQuery } from '../lib/polar_wasm_api';

import { Host } from './host';
import {
  DuplicateInstanceRegistrationError,
  InvalidCallError,
  InvalidQueryEventError,
} from './errors';

function isGenerator(x: any): x is Generator {
  return [x.next, x.return, x.throw].every(f => typeof f === 'function');
}

function isGeneratorFunction(x: any): x is GeneratorFunction {
  if (!x.constructor) return false;
  return (
    x.constructor.name === 'GeneratorFunction' ||
    isGenerator(x.constructor.prototype)
  );
}

export class Query {
  #ffiQuery: FfiQuery;
  #calls: Map<bigint, Generator>;
  #host: Host;
  results: Generator;

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

  private *start(): Generator<Map<string, any>, null, never> {
    while (true) {
      const nextEvent = this.#ffiQuery.nextEvent();
      const event: QueryEvent = parseQueryEvent(JSON.parse(nextEvent));
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

interface Result {
  bindings: Map<string, PolarValue>;
}

interface MakeExternal {
  instanceId: bigint;
  tag: string;
  fields: Map<string, PolarValue>;
}

interface ExternalCall {
  callId: bigint;
  instance: PolarValue;
  attribute: string;
  args: PolarValue[];
}

interface ExternalIsSubspecializer {
  instanceId: bigint;
  leftTag: string;
  rightTag: string;
  callId: bigint;
}

interface ExternalIsa {
  instanceId: bigint;
  tag: string;
  callId: bigint;
}

interface ExternalUnify {
  leftId: bigint;
  rightId: bigint;
  callId: bigint;
}

interface Debug {
  message: string;
}

enum QueryEventKind {
  Done,
  Result,
  MakeExternal,
  ExternalCall,
  ExternalIsSubspecializer,
  ExternalIsa,
  ExternalUnify,
  Debug,
}

interface QueryEvent {
  kind: QueryEventKind;
  data?:
    | Result
    | MakeExternal
    | ExternalCall
    | ExternalIsSubspecializer
    | ExternalIsa
    | ExternalUnify
    | Debug;
}

function parseResult(d: { [key: string]: any }): QueryEvent {
  if (
    typeof d.bindings !== 'object' ||
    Object.values(d.bindings).some(v => !isPolarValue(v))
  )
    throw new Error();
  return {
    kind: QueryEventKind.Result,
    data: { bindings: d.bindings },
  };
}

function parseMakeExternal(d: { [key: string]: any }): QueryEvent {
  if (
    !Number.isSafeInteger(d.instance_id) ||
    typeof d.instance?.tag !== 'string' ||
    typeof d.instance?.fields?.fields !== 'object' ||
    Object.values(d.instance.fields.fields).some(v => !isPolarValue(v))
  )
    throw new Error();
  return {
    kind: QueryEventKind.MakeExternal,
    data: {
      instanceId: d.instance_id as bigint,
      tag: d.instance.tag,
      fields: new Map(Object.entries(d.instance.fields.fields)),
    },
  };
}

function parseExternalCall(d: { [key: string]: any }): QueryEvent {
  if (
    !Number.isSafeInteger(d.call_id) ||
    !isPolarValue(d.instance) ||
    typeof d.attribute !== 'string' ||
    !Array.isArray(d.args) ||
    d.args.some((a: unknown) => !isPolarValue(a))
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalCall,
    data: {
      callId: d.call_id as bigint,
      instance: d.instance,
      attribute: d.attribute,
      args: d.args,
    },
  };
}

function parseExternalIsSubspecializer(d: { [key: string]: any }): QueryEvent {
  if (
    !Number.isSafeInteger(d.instance_id) ||
    !Number.isSafeInteger(d.call_id) ||
    typeof d.left_class_tag !== 'string' ||
    typeof d.right_class_tag !== 'string'
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalIsSubspecializer,
    data: {
      instanceId: d.instance_id as bigint,
      leftTag: d.left_class_tag,
      rightTag: d.right_class_tag,
      callId: d.call_id as bigint,
    },
  };
}

function parseExternalIsa(d: { [key: string]: any }): QueryEvent {
  if (
    !Number.isSafeInteger(d.instance_id) ||
    !Number.isSafeInteger(d.call_id) ||
    typeof d.class_tag !== 'string'
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalIsa,
    data: {
      instanceId: d.instance_id as bigint,
      tag: d.class_tag,
      callId: d.call_id as bigint,
    },
  };
}

function parseExternalUnify(d: { [key: string]: any }): QueryEvent {
  if (
    !Number.isSafeInteger(d.left_instance_id) ||
    !Number.isSafeInteger(d.right_instance_id) ||
    !Number.isSafeInteger(d.call_id)
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalUnify,
    data: {
      leftId: d.left_instance_id as bigint,
      rightId: d.right_instance_id as bigint,
      callId: d.call_id as bigint,
    },
  };
}

function parseDebug(d: { [key: string]: any }): QueryEvent {
  if (typeof d.message !== 'string') throw new Error();
  return {
    kind: QueryEventKind.Debug,
    data: { message: d.message },
  };
}

function parseQueryEvent(e: any): QueryEvent {
  try {
    if (typeof e?.kind !== 'string') throw new Error();
    if (e.kind === 'Done') return e;
    if (typeof e.data !== 'object') throw new Error();
    switch (e.kind) {
      case 'Result':
        return parseResult(e.data);
      case 'MakeExternal':
        return parseMakeExternal(e.data);
      case 'ExternalCall':
        return parseExternalCall(e.data);
      case 'ExternalIsSubSpecializer':
        return parseExternalIsSubspecializer(e.data);
      case 'ExternalIsa':
        return parseExternalIsa(e.data);
      case 'ExternalUnify':
        return parseExternalUnify(e.data);
      case 'Debug':
        return parseDebug(e.data);
      default:
        throw new Error();
    }
  } catch (_) {
    throw new InvalidQueryEventError(e);
  }
}
