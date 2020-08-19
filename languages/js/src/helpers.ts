import { inspect } from 'util';

import {
  InvalidQueryEventError,
  KwargsConstructorError,
  PolarError,
} from './errors';
import { isPolarTerm, QueryEventKind } from './types';
import type { obj, QueryEvent } from './types';

const root: Function = Object.getPrototypeOf(() => {});
export function ancestors(cls: Function): Function[] {
  const ancestors = [cls];
  function next(cls: Function): void {
    const parent = Object.getPrototypeOf(cls);
    if (parent === root) return;
    ancestors.push(parent);
    next(parent);
  }
  next(cls);
  return ancestors;
}

export function repr(x: any): string {
  return inspect(x);
}

export function parseQueryEvent(event: string | obj): QueryEvent {
  try {
    if (event === 'Done') return { kind: QueryEventKind.Done };
    if (typeof event === 'string') throw new Error();
    switch (true) {
      case event['Result'] !== undefined:
        return parseResult(event['Result']);
      case event['MakeExternal'] !== undefined:
        return parseMakeExternal(event['MakeExternal']);
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
        throw new PolarError('Comparing JS objects is not yet supported.');
      default:
        throw new Error();
    }
  } catch (e) {
    if (e instanceof PolarError) throw e;
    throw new InvalidQueryEventError(JSON.stringify(event));
  }
}

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

function parseMakeExternal(d: obj): QueryEvent {
  const instanceId = d.instance_id;
  const ctor = d['constructor']?.value;
  if (ctor?.InstanceLiteral !== undefined)
    throw new KwargsConstructorError(ctor?.InstanceLiteral?.tag);
  const tag = ctor?.Call?.name;
  const fields = ctor?.Call?.args;
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

function parseExternalCall({
  args,
  attribute,
  call_id: callId,
  instance,
}: obj): QueryEvent {
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

function parseDebug({ message }: obj): QueryEvent {
  if (typeof message !== 'string') throw new Error();
  return {
    kind: QueryEventKind.Debug,
    data: { message },
  };
}
