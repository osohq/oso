import {
  InvalidQueryEventError,
  KwargsConstructorError,
  PolarError,
} from './errors';
import { isPolarTerm, QueryEventKind } from './types';
import type { QueryEvent } from './types';

const root: Function = Object.getPrototypeOf(() => {});
export function ancestors(cls: Function): Function[] {
  const ancestors = [cls];
  function next(cls: Function): void {
    try {
      const parent = Object.getPrototypeOf(cls);
      if (parent === root) return;
      ancestors.push(parent);
      next(parent);
    } catch (e) {
      // TODO(gj): should it be a silent failure if something weird's in the
      // prototype chain?
      return;
    }
  }
  next(cls);
  return ancestors;
}

export function repr(x: any): string {
  return typeof x.toString === 'function' ? x.toString() : JSON.stringify(x);
}

type data = { [key: string]: any };

export function parseQueryEvent(event: string | data): QueryEvent {
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
      default:
        throw new Error();
    }
  } catch (e) {
    if (e instanceof PolarError) throw e;
    throw new InvalidQueryEventError(JSON.stringify(event));
  }
}

function parseResult({ bindings }: data): QueryEvent {
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

function parseMakeExternal(d: data): QueryEvent {
  const instanceId = d.instance_id;
  // TODO(gj): it's a little unfortunate that the property is called 'constructor'.
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
}: data): QueryEvent {
  if (
    !Number.isSafeInteger(callId) ||
    !isPolarTerm(instance) ||
    typeof attribute !== 'string' ||
    !Array.isArray(args) ||
    args.some((a: unknown) => !isPolarTerm(a))
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
}: data): QueryEvent {
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
}: data): QueryEvent {
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
}: data): QueryEvent {
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

function parseDebug({ message }: data): QueryEvent {
  if (typeof message !== 'string') throw new Error();
  return {
    kind: QueryEventKind.Debug,
    data: { message },
  };
}
