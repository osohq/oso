import {
  DuplicateClassAliasError,
  PolarError,
  UnregisteredClassError,
  UnregisteredInstanceError,
} from './errors';
import { ancestors, repr } from './helpers';
import type { Polar as FfiPolar } from './polar_wasm_api';
import { Expression } from './Expression';
import { Pattern } from './Pattern';
import { Predicate } from './Predicate';
import { Variable } from './Variable';
import type {
  Class,
  EqualityFn,
  PolarComparisonOperator,
  PolarTerm,
  PolarDictPattern,
} from './types';
import {
  Dict,
  isPolarBool,
  isPolarDict,
  isPolarExpression,
  isPolarInstance,
  isPolarList,
  isPolarNum,
  isPolarPattern,
  isPolarPredicate,
  isPolarStr,
  isPolarVariable,
} from './types';

/**
 * Translator between Polar and JavaScript.
 *
 * @internal
 */
export class Host {
  #ffiPolar: FfiPolar;
  #classes: Map<string, Class>;
  #classIds: Map<string, number>;
  clsNames: Map<Class, string>;
  #instances: Map<number, any>;
  types: Map<string, Map<string, any>>;
  fetchers: Map<string, any>;
  #equalityFn: EqualityFn;

  /**
   * Shallow clone a host to extend its state for the duration of a particular
   * query without modifying the longer-lived [[`Polar`]] host state.
   *
   * @internal
   */
  static clone(host: Host): Host {
    const clone = new Host(host.#ffiPolar, host.#equalityFn);
    clone.#classes = new Map(host.#classes);
    clone.#instances = new Map(host.#instances);
    clone.#classIds = new Map(host.#classIds);
    clone.clsNames = new Map(host.clsNames);
    clone.types = new Map(host.types);
    clone.fetchers = new Map(host.fetchers);
    return clone;
  }

  /** @internal */
  constructor(ffiPolar: FfiPolar, equalityFn: EqualityFn) {
    this.#ffiPolar = ffiPolar;
    this.#classes = new Map();
    this.#instances = new Map();
    this.#classIds = new Map();
    this.clsNames = new Map();
    this.types = new Map();
    this.fetchers = new Map();
    this.#equalityFn = equalityFn;
  }

  /**
   * Fetch a JavaScript class from the class cache.
   *
   * @param name Class name to look up.
   *
   * @internal
   */
  private getClass(name: string): Class {
    const cls = this.#classes.get(name);
    if (cls === undefined) throw new UnregisteredClassError(name);
    return cls;
  }

  /**
   * Store a JavaScript class in the class cache.
   *
   * @param cls Class to cache.
   * @param name Optional alias under which to cache the class. Defaults to the
   * class's `name` property.
   *
   * @internal
   */
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

  /**
   * Return cached instances.
   *
   * Only used by the test suite.
   *
   * @internal
   */
  instances(): any[] {
    return Array.from(this.#instances.values());
  }

  /**
   * Check if an instance exists in the instance cache.
   *
   * @internal
   */
  hasInstance(id: number): boolean {
    return this.#instances.has(id);
  }

  /**
   * Fetch a JavaScript instance from the instance cache.
   *
   * Public for the test suite.
   *
   * @internal
   */
  getInstance(id: number): any {
    if (!this.hasInstance(id)) throw new UnregisteredInstanceError(id);
    return this.#instances.get(id);
  }

  /**
   * Store a JavaScript instance in the instance cache, fetching a new instance
   * ID from the Polar VM if an ID is not provided.
   *
   * @internal
   */
  private cacheInstance(instance: any, id?: number): number {
    let instanceId = id;
    if (instanceId === undefined) {
      instanceId = this.#ffiPolar.newId();
    }
    this.#instances.set(instanceId, instance);
    return instanceId;
  }

  /**
   * Construct a JavaScript instance and store it in the instance cache.
   *
   * @internal
   */
  async makeInstance(
    name: string,
    fields: PolarTerm[],
    id: number
  ): Promise<void> {
    const cls = this.getClass(name);
    const args = await Promise.all(fields.map(f => this.toJs(f)));
    const instance = new cls(...args);
    this.cacheInstance(instance, id);
  }

  /**
   * Check if the left class is more specific than the right class with respect
   * to the given instance.
   *
   * @internal
   */
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

  /**
   * Check if the given instance is an instance of a particular class.
   *
   * @internal
   */
  async isa(polarInstance: PolarTerm, name: string): Promise<boolean> {
    const instance = await this.toJs(polarInstance);
    const cls = this.getClass(name);
    return instance instanceof cls || instance?.constructor === cls;
  }

  /**
   * Check if the given instances conform to the operator.
   *
   * @internal
   */
  async externalOp(
    op: PolarComparisonOperator,
    leftTerm: PolarTerm,
    rightTerm: PolarTerm
  ): Promise<boolean> {
    const left = await this.toJs(leftTerm);
    const right = await this.toJs(rightTerm);
    switch (op) {
      case 'Eq':
        return this.#equalityFn(left, right);
      case 'Geq':
        return left >= right;
      case 'Gt':
        return left > right;
      case 'Leq':
        return left <= right;
      case 'Lt':
        return left < right;
      case 'Neq':
        return !this.#equalityFn(left, right);
      default:
        const _: never = op;
        return _;
    }
  }

  /**
   * Turn a JavaScript value into a Polar term that's ready to be sent to the
   * Polar VM.
   *
   * @internal
   */
  toPolar(v: any): PolarTerm {
    switch (true) {
      case typeof v === 'boolean':
        return { value: { Boolean: v } };
      case Number.isInteger(v):
        return { value: { Number: { Integer: v } } };
      case typeof v === 'number':
        if (v === Infinity) {
          v = 'Infinity';
        } else if (v === -Infinity) {
          v = '-Infinity';
        } else if (Number.isNaN(v)) {
          v = 'NaN';
        }
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
      case v instanceof Expression:
        return {
          value: {
            Expression: {
              operator: v.operator,
              args: v.args.map((a: unknown) => this.toPolar(a)),
            },
          },
        };
      case v instanceof Pattern:
        const dict = this.toPolar(v.fields).value as PolarDictPattern;
        if (v.tag === undefined) {
          return { value: { Pattern: dict } };
        } else {
          return {
            value: {
              Pattern: {
                Instance: {
                  tag: v.tag,
                  fields: dict.Dictionary,
                },
              },
            },
          };
        }
      case v instanceof Dict:
        const fields = new Map(
          Object.entries(v).map(([k, v]) => [k, this.toPolar(v)])
        );
        return { value: { Dictionary: { fields } } };
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

  /**
   * Turn a Polar term from the Polar VM into a JavaScript value.
   *
   * @internal
   */
  async toJs(v: PolarTerm): Promise<any> {
    const t = v.value;
    if (isPolarStr(t)) {
      return t.String;
    } else if (isPolarNum(t)) {
      if ('Float' in t.Number) {
        const f = t.Number.Float;
        switch (f) {
          case 'Infinity':
            return Infinity;
          case '-Infinity':
            return -Infinity;
          case 'NaN':
            return NaN;
          default:
            if (typeof f !== 'number')
              throw new PolarError(
                'Expected a floating point number, got "' + f + '"'
              );
            return f;
        }
      } else {
        return t.Number.Integer;
      }
    } else if (isPolarBool(t)) {
      return t.Boolean;
    } else if (isPolarList(t)) {
      return await Promise.all(t.List.map(async el => await this.toJs(el)));
    } else if (isPolarDict(t)) {
      const valueToJs = ([k, v]: [string, PolarTerm]) =>
        this.toJs(v).then(v => [k, v]) as Promise<[string, any]>;
      const { fields } = t.Dictionary;
      const entries = await Promise.all([...fields.entries()].map(valueToJs));
      return entries.reduce((dict: Dict, [k, v]) => {
        dict[k] = v;
        return dict;
      }, new Dict());
    } else if (isPolarInstance(t)) {
      const i = this.getInstance(t.ExternalInstance.instance_id);
      return i instanceof Promise ? await i : i;
    } else if (isPolarPredicate(t)) {
      let { name, args } = t.Call;
      args = await Promise.all(args.map(a => this.toJs(a)));
      return new Predicate(name, args);
    } else if (isPolarVariable(t)) {
      return new Variable(t.Variable);
    } else if (isPolarExpression(t)) {
      // TODO(gj): Only allow expressions if the flag has been frobbed.
      const { operator, args: argTerms } = t.Expression;
      const args = await Promise.all(argTerms.map(a => this.toJs(a)));
      return new Expression(operator, args);
    } else if (isPolarPattern(t)) {
      if ('Dictionary' in t.Pattern) {
        const fields = await this.toJs({ value: t.Pattern });
        return new Pattern({ fields });
      } else {
        let {
          tag,
          fields: { fields },
        } = t.Pattern.Instance;
        const dict = await this.toJs({ value: { Dictionary: { fields } } });
        return new Pattern({ tag, fields: dict });
      }
    } else {
      const _: never = t;
      return _;
    }
  }
}
