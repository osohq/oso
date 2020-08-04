import { Polar as FfiPolar } from './wasm/polar';
import { isEqual } from 'lodash/isEqual';

const root = Object.getPrototypeOf(() => {});
function ancestors(cls: Function) {
  const ancestors = [cls];
  function next(cls: Function) {
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

class Predicate {
  readonly name: string;
  readonly args: Array<any>;

  constructor(name: string, args: Array<any>) {
    this.name = name;
    this.args = args;
  }
}

class Variable {
  readonly name: string;

  constructor(name: string) {
    this.name = name;
  }
}

interface ConstructorKwargs {
  [key: string]: any;
}

type Constructor = (kwargs: ConstructorKwargs) => object;

interface PolarStr {
  String: string;
}

function isPolarStr(v: PolarType): v is PolarStr {
  return (v as PolarStr).String !== undefined;
}

interface PolarNum {
  Number: PolarFloat | PolarInt;
}

function isPolarNum(v: PolarType): v is PolarNum {
  return (v as PolarNum).Number !== undefined;
}

interface PolarFloat {
  Float: number;
}

interface PolarInt {
  Integer: number;
}

interface PolarBool {
  Boolean: boolean;
}

function isPolarBool(v: PolarType): v is PolarBool {
  return (v as PolarBool).Boolean !== undefined;
}

interface PolarList {
  List: PolarValue[];
}

function isPolarList(v: PolarType): v is PolarList {
  return (v as PolarList).List !== undefined;
}

interface PolarDict {
  Dictionary: {
    fields: {
      [key: string]: PolarValue;
    };
  };
}

function isPolarDict(v: PolarType): v is PolarDict {
  return (v as PolarDict).Dictionary !== undefined;
}

interface PolarPredicate {
  Call: {
    name: string;
    args: PolarValue[];
  };
}

interface PolarVariable {
  Variable: {
    name: string;
  };
}

interface PolarInstance {
  ExternalInstance: {
    instance_id: bigint;
    repr: string;
  };
}

function isExternalInstance(v: PolarType): v is PolarInstance {
  return (v as PolarInstance).ExternalInstance !== undefined;
}

function isCall(v: PolarType): v is PolarPredicate {
  return (v as PolarPredicate).Call !== undefined;
}

function isVariable(v: PolarType): v is PolarVariable {
  return (v as PolarVariable).Variable !== undefined;
}

type PolarType =
  | PolarStr
  | PolarNum
  | PolarBool
  | PolarList
  | PolarDict
  | PolarPredicate
  | PolarVariable
  | PolarInstance;

interface PolarValue {
  value: PolarType;
}

interface InstanceFields {
  [key: string]: PolarValue;
}

class UnregisteredClassError extends Error {
  constructor(name: string) {
    super(`Unregistered class: ${name}.`);
    Object.setPrototypeOf(this, UnregisteredClassError.prototype);
  }
}

class UnregisteredInstanceError extends Error {
  constructor(id: bigint) {
    super(`Unregistered instance: ${id}.`);
    Object.setPrototypeOf(this, UnregisteredInstanceError.prototype);
  }
}

class MissingConstructorError extends Error {
  constructor(name: string) {
    super(`Missing constructor for class: ${name}.`);
    Object.setPrototypeOf(this, MissingConstructorError.prototype);
  }
}

class DuplicateClassAliasError extends Error {
  constructor({
    name,
    cls,
    existing,
  }: {
    name: string;
    cls: object;
    existing: object;
  }) {
    super(
      `Attempted to alias ${cls} as '${name}', but ${existing} already has that alias.`
    );
    Object.setPrototypeOf(this, DuplicateClassAliasError.prototype);
  }
}

class InvalidConstructorError extends Error {
  constructor({ constructor, cls }: { constructor: any; cls: object }) {
    super(
      `${JSON.stringify(constructor)} is not a valid constructor for ${
        cls.constructor.name
      }.`
    );
    Object.setPrototypeOf(this, InvalidConstructorError.prototype);
  }
}

class Host {
  #ffiPolar: FfiPolar;
  #classes: Map<string, Function>;
  #constructors: Map<string, Constructor>;
  #instances: Map<bigint, object>;

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
    const clsName = name === undefined ? cls.constructor.name : name;
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

  hasInstance(id: bigint): boolean {
    return this.#instances.has(id);
  }

  getInstance(id: bigint): object {
    const instance = this.#instances.get(id);
    if (instance === undefined) throw new UnregisteredInstanceError(id);
    return instance;
  }

  // NOTE(gj): BigInt requires Node >= 10.4.0
  private cacheInstance(instance: object, id?: bigint): bigint {
    let instanceId = id;
    if (instanceId === undefined) {
      instanceId = this.#ffiPolar.newId() as bigint;
    }
    this.#instances.set(instanceId, instance);
    return instanceId;
  }

  makeInstance(name: string, fields: InstanceFields, id: bigint): bigint {
    const constructor = this.getConstructor(name);
    const args = new Map(
      Object.entries(fields).map(([k, v]) => [k, this.toJs(v)])
    );
    const instance = constructor(args);
    return this.cacheInstance(instance, id);
  }

  isSubspecializer(id: bigint, left: string, right: string): boolean {
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

  isa(id: bigint, name: string): boolean {
    const instance = this.getInstance(id);
    const cls = this.getClass(name);
    return instance instanceof cls;
  }

  // TODO(gj): do more thinking about whether this should be ===
  unify(left: bigint, right: bigint): boolean {
    return isEqual(this.getInstance(left), this.getInstance(right));
  }

  toPolarTerm(v: any): PolarValue {
    switch (true) {
      case typeof v === 'boolean':
        return { value: { Boolean: v } };
      // TODO(gj): Not sure what to do here... is it cool that 5.0 becomes
      // { 'Integer': 5.0 } and that we punt on large integers? Should we
      // handle BigInts separately?
      case Number.isSafeInteger(v):
        return { value: { Number: { Integer: v } } };
      // TODO(gj): I think this roughly covers floats and excludes BigInts?
      case !Number.isInteger(v) && typeof v === 'number':
        return { value: { Number: { Float: v } } };
      case typeof v === 'string':
        return { value: { String: v } };
      // TODO(gj): do we want to handle TypedArrays here with
      // ArrayBuffer.isView(v)?
      case Array.isArray(v):
        return { value: { List: v.map((el: any) => this.toPolarTerm(el)) } };
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
              args: v.args.map((el: any) => this.toPolarTerm(el)),
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

  // TODO(gj): handle Set?
  private toJs(v: PolarValue): any {
    const t = v.value;
    if (isPolarStr(t)) {
      return t.String;
    } else if (isPolarNum(t)) {
      if ('Float' in t.Number) {
        return t.Number.Float;
      } else {
        // TODO(gj): handle BigInts?
        return t.Number.Integer;
      }
    } else if (isPolarBool(t)) {
      return t.Boolean;
    } else if (isPolarList(t)) {
      return t.List.map(this.toJs);
      // TODO(gj): handle Map?
    } else if (isPolarDict(t)) {
      const copy = {};
      Object.entries(t.Dictionary.fields).forEach(
        ([k, v]) => (copy[k] = this.toJs(v))
      );
      return copy;
    } else if (isExternalInstance(t)) {
      return this.getInstance(t.ExternalInstance.instance_id);
    } else if (isCall(t)) {
      return new Predicate(
        t.Call.name,
        t.Call.args.map((a) => this.toJs(a))
      );
    } else if (isVariable(t)) {
      return new Variable(t.Variable.name);
    } else {
      // TODO(gj): assert unreachable
    }
  }
}
