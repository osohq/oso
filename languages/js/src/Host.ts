import isEqual from 'lodash/isEqual';

import {
  DuplicateClassAliasError,
  UnregisteredClassError,
  UnregisteredInstanceError,
} from './errors';
import { ancestors } from './helpers';
import type { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import { Variable } from './Variable';
import type { Class, PolarValue } from './types';
import {
  isPolarStr,
  isPolarNum,
  isPolarBool,
  isPolarList,
  isPolarDict,
  isPolarInstance,
  isPolarPredicate,
  isPolarVariable,
} from './types';

const GLOBAL_BUILTIN_OBJECTS = Object.getOwnPropertyNames(global)
  .map(name => Reflect.get(global, name))
  .filter(prop => typeof prop === 'object');

export class Host {
  #ffiPolar: FfiPolar;
  #classes: Map<string, Class>;
  #instances: Map<number, any>;

  static clone(host: Host): Host {
    const clone = new Host(host.#ffiPolar);
    clone.#classes = new Map(host.#classes);
    clone.#instances = new Map(host.#instances);
    return clone;
  }

  constructor(ffiPolar: FfiPolar) {
    this.#ffiPolar = ffiPolar;
    this.#classes = new Map();
    this.#instances = new Map();
  }

  private getClass(name: string): Class {
    const cls = this.#classes.get(name);
    if (cls === undefined) throw new UnregisteredClassError(name);
    return cls;
  }

  cacheClass<T>(cls: Class<T>, name?: string): string {
    const clsName = name === undefined ? cls.name : name;
    console.assert(clsName, cls.toString());
    const existing = this.#classes.get(clsName);
    if (existing !== undefined)
      throw new DuplicateClassAliasError({
        name: clsName,
        cls,
        existing,
      });
    this.#classes.set(clsName, cls);
    return clsName;
  }

  hasInstance(id: number): boolean {
    return this.#instances.has(id);
  }

  private getInstance(id: number): object {
    const instance = this.#instances.get(id);
    if (instance === undefined) throw new UnregisteredInstanceError(id);
    return instance;
  }

  private cacheInstance(instance: any, id?: number): number {
    let instanceId = id;
    if (instanceId === undefined) {
      instanceId = this.#ffiPolar.newId();
    }
    this.#instances.set(instanceId, instance);
    return instanceId;
  }

  makeInstance(name: string, fields: PolarValue[], id: number): number {
    const cls = this.getClass(name);
    const args = fields.map(f => this.toJs(f));
    const instance = new cls(...args);
    return this.cacheInstance(instance, id);
  }

  isSubspecializer(id: number, left: string, right: string): boolean {
    const instance = this.getInstance(id);
    const mro = ancestors(instance.constructor);
    const leftIndex = mro.indexOf(this.getClass(left));
    const rightIndex = mro.indexOf(this.getClass(right));
    if (leftIndex === -1) {
      return false;
    } else if (rightIndex === -1) {
      return true;
    } else {
      return leftIndex < rightIndex;
    }
  }

  isa(instance: PolarValue, name: string): boolean {
    const jsInstance = this.toJs(instance);
    const cls = this.getClass(name);
    // TODO(gj): is this correct?
    return jsInstance instanceof cls || jsInstance.constructor === cls;
  }

  unify(left: number, right: number): boolean {
    return isEqual(this.getInstance(left), this.getInstance(right));
  }

  // TODO(gj): should PolarValue be called PolarTerm?
  toPolarTerm(v: any): PolarValue {
    switch (true) {
      case typeof v === 'boolean':
        return { value: { Boolean: v } };
      case Number.isInteger(v):
        return { value: { Number: { Integer: v } } };
      // TODO(gj): Handle Infinity, -Infinity, -0, NaN, etc.
      case typeof v === 'number':
        return { value: { Number: { Float: v } } };
      case typeof v === 'string':
        return { value: { String: v } };
      case Array.isArray(v):
        return {
          value: { List: v.map((el: unknown) => this.toPolarTerm(el)) },
        };
      case v instanceof Predicate:
        return {
          value: {
            Call: {
              name: v.name,
              args: v.args.map((el: unknown) => this.toPolarTerm(el)),
            },
          },
        };
      case v instanceof Variable:
        return { value: { Variable: v } };
      // TODO(gj): is this the best way to determine whether it's an object?
      // TODO(gj): should we handle Maps here?
      // TODO(gj): Need to find a better way to filter out Math.
      case typeof v === 'object' &&
        v.constructor.prototype === {}.constructor.prototype &&
        !GLOBAL_BUILTIN_OBJECTS.includes(v):
        return {
          value: {
            Dictionary: {
              fields: Object.assign(
                {},
                ...Object.entries(v).map(([k, v]) => ({
                  [k]: this.toPolarTerm(v),
                }))
              ),
            },
          },
        };
      default:
        return {
          value: {
            ExternalInstance: {
              instance_id: this.cacheInstance(v),
              repr: JSON.stringify(v),
            },
          },
        };
    }
  }

  toJs(v: PolarValue): any {
    const t = v.value;
    if (isPolarStr(t)) {
      return t.String;
    } else if (isPolarNum(t)) {
      if ('Float' in t.Number) {
        return t.Number.Float;
      } else {
        return t.Number.Integer;
      }
    } else if (isPolarBool(t)) {
      return t.Boolean;
    } else if (isPolarList(t)) {
      return t.List.map(this.toJs);
      // TODO(gj): handle Map?
    } else if (isPolarDict(t)) {
      return Object.assign(
        {},
        ...Object.entries(t.Dictionary.fields).map(([k, v]) => ({
          [k]: this.toPolarTerm(v),
        }))
      );
    } else if (isPolarInstance(t)) {
      return this.getInstance(t.ExternalInstance.instance_id);
    } else if (isPolarPredicate(t)) {
      return new Predicate(
        t.Call.name,
        t.Call.args.map(a => this.toJs(a))
      );
    } else if (isPolarVariable(t)) {
      return new Variable(t.Variable.name);
    }
  }
}
