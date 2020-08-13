import isEqual from 'lodash/isEqual';

import {
  DuplicateClassAliasError,
  UnregisteredClassError,
  UnregisteredInstanceError,
} from './errors';
import { ancestors, repr } from './helpers';
import type { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import { Variable } from './Variable';
import type { Class, PolarTerm } from './types';
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

  // Public for the test suite.
  getInstance(id: number): object {
    const instance = this.#instances.get(id);
    if (instance === undefined) throw new UnregisteredInstanceError(id);
    return instance;
  }

  cacheInstance(instance: any, id?: number): number {
    let instanceId = id;
    if (instanceId === undefined) {
      instanceId = this.#ffiPolar.newId();
    }
    this.#instances.set(instanceId, instance);
    return instanceId;
  }

  makeInstance(name: string, fields: PolarTerm[], id: number): number {
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

  isa(instance: PolarTerm, name: string): boolean {
    const jsInstance = this.toJs(instance);
    const cls = this.getClass(name);
    return jsInstance instanceof cls || jsInstance.constructor === cls;
  }

  unify(left: number, right: number): boolean {
    return isEqual(this.getInstance(left), this.getInstance(right));
  }

  toPolarTerm(v: any): PolarTerm {
    switch (true) {
      case typeof v === 'boolean':
        return { value: { Boolean: v } };
      case Number.isInteger(v):
        return { value: { Number: { Integer: v } } };
      case typeof v === 'number' && Number.isFinite(v):
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
        return { value: { Variable: v.name } };
      // JS global built-ins like Math are hard to check for.
      case typeof v === 'object' &&
        v.constructor.prototype === {}.constructor.prototype:
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
              repr: repr(v),
            },
          },
        };
    }
  }

  toJs(v: PolarTerm): any {
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
    } else if (isPolarDict(t)) {
      const { fields } = t.Dictionary;
      let entries;
      // TODO(gj): Why is this sometimes a Map and sometimes an Object?
      if (typeof fields.entries === 'function') {
        entries = [...fields.entries()];
      } else {
        entries = Object.entries(fields);
      }
      return entries.reduce((obj: { [key: string]: any }, [k, v]) => {
        obj[k] = this.toJs(v);
        return obj;
      }, {});
    } else if (isPolarInstance(t)) {
      return this.getInstance(t.ExternalInstance.instance_id);
    } else if (isPolarPredicate(t)) {
      return new Predicate(
        t.Call.name,
        t.Call.args.map(a => this.toJs(a))
      );
    } else if (isPolarVariable(t)) {
      return new Variable(t.Variable);
    }
  }
}
