import {
  InvalidQueryEventError,
  KwargsConstructorError,
  PolarError,
} from './errors';
import { isPolarValue, QueryEventKind } from './types';
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

export function parseQueryEvent(
  event: string | { [key: string]: any }
): QueryEvent {
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
  const id = d.instance_id;
  const ctor = d['constructor']?.value;
  if (ctor?.InstanceLiteral !== undefined)
    throw new KwargsConstructorError(ctor?.InstanceLiteral?.tag);
  if (
    !Number.isSafeInteger(id) ||
    typeof ctor?.Call?.name !== 'string' ||
    !Array.isArray(ctor?.Call?.args) ||
    ctor.Call.args.some((v: unknown) => !isPolarValue(v))
  )
    throw new Error();
  return {
    kind: QueryEventKind.MakeExternal,
    data: {
      instanceId: id,
      tag: ctor.Call.name,
      fields: ctor.Call.args,
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
      callId: d.call_id,
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
      instanceId: d.instance_id,
      leftTag: d.left_class_tag,
      rightTag: d.right_class_tag,
      callId: d.call_id,
    },
  };
}

function parseExternalIsa(d: { [key: string]: any }): QueryEvent {
  const callId = d?.call_id;
  // const instanceId = d?.instance?.value?.ExternalInstance?.instance_id;
  const instance = d?.instance;
  const tag = d?.class_tag;
  if (
    !Number.isSafeInteger(callId) ||
    !isPolarValue(instance) ||
    typeof tag !== 'string'
  )
    throw new Error();
  return {
    kind: QueryEventKind.ExternalIsa,
    data: { instance, tag, callId },
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
      leftId: d.left_instance_id,
      rightId: d.right_instance_id,
      callId: d.call_id,
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
