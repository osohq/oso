import { inspect } from 'util';

const _readFile = require('fs')?.readFile;

import { InvalidQueryEventError, KwargsError, PolarError } from './errors';
import {
  isPolarComparisonOperator,
  isPolarPredicate,
  isPolarTerm,
  QueryEventKind,
} from './types';
import type { Class, obj, PolarTerm, QueryEvent } from './types';

/**
 * Assemble the prototypal inheritance chain of a class.
 *
 * @returns The inheritance chain as a list of prototypes in most-to-least
 * specific order.
 *
 * @internal
 */
export function ancestors(cls: Function): Function[] {
  const ancestors = [cls];
  function next(cls: Function): void {
    const parent = Object.getPrototypeOf(cls);
    if (parent === Function.prototype) return;
    ancestors.push(parent);
    next(parent);
  }
  next(cls);
  return ancestors;
}

/**
 * Stringify a value.
 *
 * @returns A string representation of the input value.
 *
 * @internal
 */
export function repr(x: unknown): string {
  return inspect(x);
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary
 * into a valid [[`QueryEvent`]].
 *
 * @internal
 */
export function parseQueryEvent(event: string | obj): QueryEvent {
  try {
    if (isString(event)) throw new Error();
    switch (true) {
      case event['Done'] !== undefined:
        return { kind: QueryEventKind.Done };
      case event['Result'] !== undefined:
        return parseResult(event['Result']);
      case event['MakeExternal'] !== undefined:
        return parseMakeExternal(event['MakeExternal']);
      case event['NextExternal'] !== undefined:
        return parseNextExternal(event['NextExternal']);
      case event['ExternalCall'] !== undefined:
        return parseExternalCall(event['ExternalCall']);
      case event['ExternalIsSubSpecializer'] !== undefined:
        return parseExternalIsSubspecializer(event['ExternalIsSubSpecializer']);
      case event['ExternalIsSubclass'] !== undefined:
        return parseExternalIsSubclass(event['ExternalIsSubclass']);
      case event['ExternalIsa'] !== undefined:
        return parseExternalIsa(event['ExternalIsa']);
      case event['ExternalIsaWithPath'] !== undefined:
        return parseExternalIsaWithPath(event['ExternalIsaWithPath']);
      case event['Debug'] !== undefined:
        return parseDebug(event['Debug']);
      case event['ExternalOp'] !== undefined:
        return parseExternalOp(event['ExternalOp']);
      default:
        throw new Error();
    }
  } catch (e) {
    if (e instanceof PolarError) throw e;
    throw new InvalidQueryEventError(JSON.stringify(event));
  }
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`Result`]].
 *
 * @internal
 */
function parseResult(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const { bindings } = event;
  if (!isMapOfPolarTerms(bindings)) throw new Error();
  return { kind: QueryEventKind.Result, data: { bindings } };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`MakeExternal`]].
 *
 * @internal
 */
function parseMakeExternal(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const { instance_id: instanceId } = event;
  if (!isSafeInteger(instanceId)) throw new Error();
  const ctor = event['constructor'];
  if (!isPolarTerm(ctor)) throw new Error();
  if (!isPolarPredicate(ctor.value)) throw new Error();
  // TODO(gj): can we remove this kwargs check?
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (ctor.value.Call.kwargs) throw new KwargsError();
  const { name: tag, args: fields } = ctor.value.Call;
  if (!isString(tag)) throw new Error();
  if (!isArrayOf(fields, isPolarTerm)) throw new Error();
  return {
    kind: QueryEventKind.MakeExternal,
    data: { fields, instanceId, tag },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`NextExternal`]].
 *
 * @internal
 */
function parseNextExternal(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const { call_id: callId, iterable } = event;
  if (!isSafeInteger(callId)) throw new Error();
  if (!isPolarTerm(iterable)) throw new Error();
  return { kind: QueryEventKind.NextExternal, data: { callId, iterable } };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalCall`]].
 *
 * @internal
 */
function parseExternalCall(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const { args, kwargs, attribute, call_id: callId, instance } = event;
  if (args !== undefined && !isArrayOf(args, isPolarTerm)) throw new Error();
  if (kwargs) throw new KwargsError();
  if (!isString(attribute)) throw new Error();
  if (!isSafeInteger(callId)) throw new Error();
  if (!isPolarTerm(instance)) throw new Error();
  return {
    kind: QueryEventKind.ExternalCall,
    data: { args, attribute, callId, instance },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsSubspecializer`]].
 *
 * @internal
 */
function parseExternalIsSubspecializer(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const {
    call_id: callId,
    instance_id: instanceId,
    left_class_tag: leftTag,
    right_class_tag: rightTag,
  } = event;
  if (!isSafeInteger(callId)) throw new Error();
  if (!isSafeInteger(instanceId)) throw new Error();
  if (!isString(leftTag)) throw new Error();
  if (!isString(rightTag)) throw new Error();
  return {
    kind: QueryEventKind.ExternalIsSubspecializer,
    data: { callId, instanceId, leftTag, rightTag },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsSubclass`]].
 *
 * @internal
 */
function parseExternalIsSubclass(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const {
    call_id: callId,
    left_class_tag: leftTag,
    right_class_tag: rightTag,
  } = event;
  if (!isSafeInteger(callId)) throw new Error();
  if (!isString(leftTag)) throw new Error();
  if (!isString(rightTag)) throw new Error();
  return {
    kind: QueryEventKind.ExternalIsSubclass,
    data: { callId, leftTag, rightTag },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsa`]].
 *
 * @internal
 */
function parseExternalIsa(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const { call_id: callId, instance, class_tag: tag } = event;
  if (!isSafeInteger(callId)) throw new Error();
  if (!isPolarTerm(instance)) throw new Error();
  if (!isString(tag)) throw new Error();
  return { kind: QueryEventKind.ExternalIsa, data: { callId, instance, tag } };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsa`]].
 *
 * @internal
 */
function parseExternalIsaWithPath(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const {
    path,
    call_id: callId,
    base_tag: baseTag,
    class_tag: classTag,
  } = event;
  if (!isSafeInteger(callId)) throw new Error();
  if (!isString(baseTag)) throw new Error();
  if (!isString(classTag)) throw new Error();
  if (!isArrayOf(path, isPolarTerm)) throw new Error();
  return {
    kind: QueryEventKind.ExternalIsaWithPath,
    data: { callId, baseTag, path, classTag },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalOp`]].
 *
 * @internal
 */
function parseExternalOp(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const { call_id: callId, args, operator } = event;
  if (!isSafeInteger(callId)) throw new Error();
  if (!isArrayOf(args, isPolarTerm) || args.length !== 2) throw new Error();
  if (!isString(operator)) throw new Error();
  if (!isPolarComparisonOperator(operator))
    throw new PolarError(
      `Unsupported external operation '${repr(args[0])} ${operator} ${repr(
        args[1]
      )}'`
    );
  return { kind: QueryEventKind.ExternalOp, data: { args, callId, operator } };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`Debug`]].
 *
 * @internal
 */
function parseDebug(event: unknown): QueryEvent {
  if (!isObj(event)) throw new Error();
  const { message } = event;
  if (!isString(message)) throw new Error();
  return { kind: QueryEventKind.Debug, data: { message } };
}

/**
 * Promisified version of the pre-`fs/promises` asynchronous `fs.readFile`
 * function since none of the following work on all Node.js versions we want to
 * support (>= 10):
 *
 * ```ts
 * import { readFile } from 'fs/promises';
 * import { promises } from 'fs';
 * const { readFile } = require('fs/promises');
 * ```
 *
 * @internal
 */
export function readFile(file: string): Promise<string> {
  return new Promise((res, rej) =>
    _readFile!(file, { encoding: 'utf8' }, (err: string, contents: string) =>
      err === null ? res(contents) : rej(err)
    )
  );
}

// Optional ANSI escape sequences for the REPL.
let RESET = '';
let FG_BLUE = '';
let FG_RED = '';
if (
  typeof process?.stdout?.getColorDepth === 'function' &&
  process.stdout.getColorDepth() >= 4 &&
  typeof process?.stderr?.getColorDepth === 'function' &&
  process.stderr.getColorDepth() >= 4
) {
  RESET = '\x1b[0m';
  FG_BLUE = '\x1b[34m';
  FG_RED = '\x1b[31m';
}
/** @internal */
export const PROMPT = FG_BLUE + 'query> ' + RESET;

/** @internal */
export function printError(e: Error) {
  console.error(FG_RED + e.name + RESET);
  console.error(e.message);
}

/**
 * https://stackoverflow.com/a/46759625
 *
 * @internal
 */
export function isConstructor(f: unknown): f is Class {
  try {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    Reflect.construct(String, [], f);
    return true;
  } catch (e) {
    return false;
  }
}

/**
 * Type guard to test if a value is a [[`obj`]].
 *
 * @internal
 */
export function isObj(x: unknown): x is obj {
  return typeof x === 'object' && x !== null;
}

/**
 * Type guard to test if `x` is a `string`.
 *
 * @internal
 */
export const isString = (x: unknown): x is string => typeof x === 'string';

/**
 * Type guard to test if a value is an ES6 Map with string keys and PolarTerm
 * values.
 *
 * @internal
 */
const isMapOfPolarTerms = (x: unknown): x is Map<string, PolarTerm> =>
  x instanceof Map &&
  [...x.keys()].every(isString) &&
  [...x.values()].every(isPolarTerm);

/**
 * Type guard to test if `x` is an `Array` where every member matches a
 * type-narrowing predicate `p`.
 *
 * @internal
 */
type Pred<T> = (x: unknown) => x is T;
const isArrayOf = <T>(x: unknown, p: Pred<T>): x is Array<T> =>
  Array.isArray(x) && x.every(p);

/**
 * Type guard to test if a value is a safe integer.
 *
 * @internal
 */
function isSafeInteger(x: unknown): x is number {
  return Number.isSafeInteger(x);
}

/**
 * Promisify a 1-arity function.
 */
export const promisify1 =
  <A, B>(f: (a: A) => B) =>
  (a: A) =>
    Promise.resolve(f(a));
