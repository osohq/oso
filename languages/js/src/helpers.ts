import { InvalidQueryEventError } from './errors';
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
  e: string | { [key: string]: any }
): QueryEvent {
  try {
    if (e === 'Done') return { kind: QueryEventKind.Debug };
    if (typeof e === 'string') throw new Error();
    switch (true) {
      case e['Result'] !== undefined:
        return parseResult(e['Result']);
      case e['MakeExternal'] !== undefined:
        return parseMakeExternal(e['MakeExternal']);
      case e['ExternalCall'] !== undefined:
        return parseExternalCall(e['ExternalCall']);
      case e['ExternalIsSubSpecializer'] !== undefined:
        return parseExternalIsSubspecializer(e['ExternalIsSubSpecializer']);
      case e['ExternalIsa'] !== undefined:
        return parseExternalIsa(e['ExternalIsa']);
      case e['ExternalUnify'] !== undefined:
        return parseExternalUnify(e['ExternalUnify']);
      case e['Debug'] !== undefined:
        return parseDebug(e['Debug']);
      default:
        throw new Error();
    }
  } catch (_) {
    throw new InvalidQueryEventError(JSON.stringify(e));
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
