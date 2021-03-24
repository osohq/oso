import { inspect } from 'util';

const _readFile = require('fs')?.readFile;

import { InvalidQueryEventError, KwargsError, PolarError } from './errors';
import { isPolarTerm, isPolarOperator, QueryEventKind } from './types';
import type { obj, QueryEvent } from './types';

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
export function repr(x: any): string {
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
    if (typeof event === 'string') throw new Error();
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
      case event['ExternalIsa'] !== undefined:
        return parseExternalIsa(event['ExternalIsa']);
      case event['ExternalUnify'] !== undefined:
        return parseExternalUnify(event['ExternalUnify']);
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
function parseResult({ bindings }: obj): QueryEvent {
  if (
    typeof bindings !== 'object' ||
    Object.values(bindings).some(v => !isPolarTerm(v))
  )
    throw new Error();
  return {
    kind: QueryEventKind.Result,
    data: { bindings },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`MakeExternal`]].
 *
 * @internal
 */
function parseMakeExternal(d: obj): QueryEvent {
  const instanceId = d.instance_id;
  const ctor = d['constructor']?.value?.Call;
  const tag = ctor?.name;
  const fields = ctor?.args;
  if (ctor?.kwargs) throw new KwargsError();
  if (
    !Number.isSafeInteger(instanceId) ||
    typeof tag !== 'string' ||
    !Array.isArray(fields) ||
    fields.some((v: unknown) => !isPolarTerm(v))
  )
    throw new Error();
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
function parseNextExternal(d: obj): QueryEvent {
  const callId = d.call_id;
  const iterable = d.iterable;
  if (!Number.isSafeInteger(callId) || !isPolarTerm(iterable))
    throw new Error();
  return {
    kind: QueryEventKind.NextExternal,
    data: { callId, iterable },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalCall`]].
 *
 * @internal
 */
function parseExternalCall({
  args,
  kwargs,
  attribute,
  call_id: callId,
  instance,
}: obj): QueryEvent {
  if (kwargs) throw new KwargsError();
  if (
    !Number.isSafeInteger(callId) ||
    !isPolarTerm(instance) ||
    typeof attribute !== 'string' ||
    (args !== undefined &&
      (!Array.isArray(args) || args.some((a: unknown) => !isPolarTerm(a))))
  )
    throw new Error();
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
function parseExternalIsSubspecializer({
  call_id: callId,
  instance_id: instanceId,
  left_class_tag: leftTag,
  right_class_tag: rightTag,
}: obj): QueryEvent {
  if (
    !Number.isSafeInteger(instanceId) ||
    !Number.isSafeInteger(callId) ||
    typeof leftTag !== 'string' ||
    typeof rightTag !== 'string'
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalIsSubspecializer,
    data: { callId, instanceId, leftTag, rightTag },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalIsa`]].
 *
 * @internal
 */
function parseExternalIsa({
  call_id: callId,
  instance,
  class_tag: tag,
}: obj): QueryEvent {
  if (
    !Number.isSafeInteger(callId) ||
    !isPolarTerm(instance) ||
    typeof tag !== 'string'
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalIsa,
    data: { callId, instance, tag },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalOp`]].
 *
 * @internal
 */
function parseExternalOp({ call_id: callId, args, operator }: obj): QueryEvent {
  if (
    !Number.isSafeInteger(callId) ||
    (args !== undefined &&
      (!Array.isArray(args) ||
        args.length !== 2 ||
        args.some((a: unknown) => !isPolarTerm(a))))
  )
    throw new Error();
  if (!isPolarOperator(operator))
    throw new PolarError(
      `Unsupported external operation '${repr(args[0])} ${operator} ${repr(
        args[1]
      )}'`
    );
  return {
    kind: QueryEventKind.ExternalOp,
    data: {
      args,
      callId,
      operator,
    },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * an [[`ExternalUnify`]].
 *
 * @internal
 */
function parseExternalUnify({
  call_id: callId,
  left_instance_id: leftId,
  right_instance_id: rightId,
}: obj): QueryEvent {
  if (
    !Number.isSafeInteger(callId) ||
    !Number.isSafeInteger(leftId) ||
    !Number.isSafeInteger(rightId)
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalUnify,
    data: { callId, leftId, rightId },
  };
}

/**
 * Try to parse a JSON payload received from across the WebAssembly boundary as
 * a [[`Debug`]].
 *
 * @internal
 */
function parseDebug({ message }: obj): QueryEvent {
  if (typeof message !== 'string') throw new Error();
  return {
    kind: QueryEventKind.Debug,
    data: { message },
  };
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
export function isConstructor(f: unknown): boolean {
  try {
    Reflect.construct(String, [], f);
    return true;
  } catch (e) {
    return false;
  }
}
