import {
  DuplicateClassAliasError,
  UnregisteredClassError,
  UnregisteredInstanceError,
} from './errors';
import { ancestors, repr } from './helpers';
import type { Polar as FfiPolar } from './polar_wasm_api';
import { Predicate } from './Predicate';
import { Variable } from './Variable';
import type { Class, EqualityFn, obj, PolarTerm } from './types';
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
  #equalityFn: EqualityFn;

  static clone(host: Host): Host {
    const clone = new Host(host.#ffiPolar, host.#equalityFn);
    clone.#classes = new Map(host.#classes);
    clone.#instances = new Map(host.#instances);
    return clone;
  }

  constructor(ffiPolar: FfiPolar, equalityFn: EqualityFn) {
    this.#ffiPolar = ffiPolar;
    this.#classes = new Map();
    this.#instances = new Map();
    this.#equalityFn = equalityFn;
  }

  private getClass(name: string): Class {
    const cls = this.#classes.get(name);
    if (cls === undefined) throw new UnregisteredClassError(name);
    return cls;
  }

  cacheClass<T>(cls: Class<T>, name?: string): string {
    const clsName = name === undefined ? cls.name : name;
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
  getInstance(id: number): any {
    if (!this.hasInstance(id)) throw new UnregisteredInstanceError(id);
    return this.#instances.get(id);
  }

  private cacheInstance(instance: any, id?: number): number {
    let instanceId = id;
    if (instanceId === undefined) {
      instanceId = this.#ffiPolar.newId();
    }
    this.#instances.set(instanceId, instance);
    return instanceId;
  }

  // Return value only used in tests.
  async makeInstance(
    name: string,
    fields: PolarTerm[],
    id: number
  ): Promise<number> {
    const cls = this.getClass(name);
    const args = await Promise.all(fields.map(async f => await this.toJs(f)));
    const instance = new cls(...args);
    return this.cacheInstance(instance, id);
  }

  async isSubspecializer(
    id: number,
    left: string,
    right: string
  ): Promise<boolean> {
    let instance = this.getInstance(id);
    instance = instance instanceof Promise ? await instance : instance;
    if (!(instance?.constructor instanceof Function)) return false;
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

  async isa(polarInstance: PolarTerm, name: string): Promise<boolean> {
    const instance = await this.toJs(polarInstance);
    const cls = this.getClass(name);
    return instance instanceof cls || instance?.constructor === cls;
  }

  async unify(leftId: number, rightId: number): Promise<boolean> {
    let left = this.getInstance(leftId);
    let right = this.getInstance(rightId);
    left = left instanceof Promise ? await left : left;
    right = right instanceof Promise ? await right : right;
    return this.#equalityFn(left, right);
  }

  toPolar(v: any): PolarTerm {
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
        return { value: { List: v.map((el: unknown) => this.toPolar(el)) } };
      case v instanceof Predicate:
        const args = v.args.map((el: unknown) => this.toPolar(el));
        return { value: { Call: { name: v.name, args } } };
      case v instanceof Variable:
        return { value: { Variable: v.name } };
      default:
        const instance_id = this.cacheInstance(v);
        return {
          value: {
            ExternalInstance: {
              instance_id,
              repr: repr(v),
              constructor: undefined,
            },
          },
        };
    }
  }

  async toJs(v: PolarTerm): Promise<any> {
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
      return await Promise.all(t.List.map(async el => await this.toJs(el)));
    } else if (isPolarDict(t)) {
      const { fields } = t.Dictionary;
      let entries =
        typeof fields.entries === 'function'
          ? Array.from(fields.entries())
          : Object.entries(fields);
      entries = await Promise.all(
        entries.map(async ([k, v]) => [k, await this.toJs(v)]) as Promise<
          [string, any]
        >[]
      );
      return entries.reduce((obj: obj, [k, v]) => {
        obj[k] = v;
        return obj;
      }, {});
    } else if (isPolarInstance(t)) {
      const i = this.getInstance(t.ExternalInstance.instance_id);
      return i instanceof Promise ? await i : i;
    } else if (isPolarPredicate(t)) {
      let { name, args } = t.Call;
      args = await Promise.all(args.map(async a => await this.toJs(a)));
      return new Predicate(name, args);
    } else if (isPolarVariable(t)) {
      return new Variable(t.Variable);
    }
  }
}
