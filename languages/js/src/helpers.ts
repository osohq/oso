import { InvalidQueryEventError } from './errors';

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

export function parseQueryEvent(e: any): QueryEvent {
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
