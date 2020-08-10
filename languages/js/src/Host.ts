import isEqual from 'lodash/isEqual';

import {
  DuplicateClassAliasError,
  InvalidConstructorError,
  MissingConstructorError,
  UnregisteredClassError,
  UnregisteredInstanceError,
} from './errors';
import { ancestors } from './helpers';
import type { Polar as FfiPolar } from '../dist/polar_wasm_api';
import { Predicate } from './Predicate';
import { Variable } from './Variable';
import type { Constructor, ConstructorKwargs, PolarValue } from './types';
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
  #classes: Map<string, Function>;
  #constructors: Map<string, Constructor>;
  #instances: Map<number, object>;

  constructor(ffiPolar: FfiPolar) {
    this.#ffiPolar = ffiPolar;
    this.#classes = new Map();
    this.#constructors = new Map();
    this.#instances = new Map();
  }

  dup(): Host {
    return { ...this };
  }

  private getClass(name: string): Function {
    const cls = this.#classes.get(name);
    if (cls === undefined) throw new UnregisteredClassError(name);
    return cls;
  }

  cacheClass(cls: Function, name?: string, constructor?: Constructor): string {
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
    let ctor: Constructor;
    switch (typeof constructor) {
      case 'undefined':
        ctor = (kwargs: ConstructorKwargs) => Reflect.construct(cls, [kwargs]);
        break;
      case 'function':
        ctor = (kwargs: ConstructorKwargs) =>
          Reflect.apply(constructor, cls, [kwargs]);
        break;
      case 'string':
        const prop = Reflect.get(cls, constructor);
        if (prop === undefined) {
          throw new InvalidConstructorError({ constructor, cls });
        } else {
          ctor = (kwargs: ConstructorKwargs) =>
            Reflect.apply(prop, cls, [kwargs]);
          break;
        }
      default:
        throw new InvalidConstructorError({ constructor, cls });
    }
    this.#constructors.set(clsName, ctor);
    return clsName;
  }

  private getConstructor(name: string): Constructor {
    const constructor = this.#constructors.get(name);
    if (constructor === undefined) throw new MissingConstructorError(name);
    return constructor;
  }

  hasInstance(id: number): boolean {
    return this.#instances.has(id);
  }

  getInstance(id: number): object {
    const instance = this.#instances.get(id);
    if (instance === undefined) throw new UnregisteredInstanceError(id);
    return instance;
  }

  private cacheInstance(instance: object, id?: number): number {
    let instanceId = id;
    if (instanceId === undefined) {
      instanceId = this.#ffiPolar.newId();
    }
    this.#instances.set(instanceId, instance);
    return instanceId;
  }

  makeInstance(
    name: string,
    fields: Map<string, PolarValue>,
    id: number
  ): number {
    const constructor = this.getConstructor(name);
    const args = new Map(
      Object.entries(fields).map(([k, v]) => [k, this.toJs(v)])
    );
    const instance = constructor(args);
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

  isa(id: number, name: string): boolean {
    const instance = this.getInstance(id);
    const cls = this.getClass(name);
    // TODO(gj): is this correct?
    return instance instanceof cls || instance.constructor === cls;
  }

  // TODO(gj): do more thinking about whether this should be ===
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
      // TODO(gj): is this the best way to determine whether it's an object?
      // TODO(gj): should we handle Maps here?
      case v.constructor === Object:
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
