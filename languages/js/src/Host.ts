import {
  DataFilteringConfigurationError,
  DuplicateClassAliasError,
  InvalidConstructorError,
  PolarError,
  UnregisteredClassError,
  UnregisteredInstanceError,
} from './errors';
import {
  ancestors,
  isConstructor,
  isObj,
  isString,
  promisify1,
  repr,
} from './helpers';
import type { Polar as FfiPolar } from './polar_wasm_api';
import { Expression } from './Expression';
import { Pattern } from './Pattern';
import { Predicate } from './Predicate';
import { Variable } from './Variable';
import type {
  Class,
  ClassParams,
  EqualityFn,
  PolarComparisonOperator,
  PolarTerm,
  UserTypeParams,
  BuildQueryFn,
  ExecQueryFn,
  CombineQueryFn,
  DataFilteringQueryParams,
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
import { Relation } from './dataFiltering';
import type { SerializedFields } from './dataFiltering';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class UserType<Type extends Class<T>, T = any, Query = any> {
  name: string;
  cls: Type;
  id: number;
  fields: Map<string, Class | Relation>;
  buildQuery: BuildQueryFn<Promise<Query>>;
  execQuery: ExecQueryFn<Query, Promise<T[]>>;
  combineQuery: CombineQueryFn<Query>;

  constructor({
    name,
    cls,
    id,
    fields,
    buildQuery,
    execQuery,
    combineQuery,
  }: UserTypeParams<Type>) {
    this.name = name;
    this.cls = cls;
    this.fields = fields;
    // NOTE(gj): these `promisify1()` calls are for Promisifying synchronous
    // return values from {build,exec,combine}Query. Since a user's
    // implementation *might* return a Promise, we want to `await` _all_
    // invocations.
    this.buildQuery = promisify1(buildQuery);
    this.execQuery = promisify1(execQuery);
    this.combineQuery = combineQuery;
    this.id = id;
  }
}

/**
 * Translator between Polar and JavaScript.
 *
 * @internal
 */
export class Host implements Required<DataFilteringQueryParams> {
  #ffiPolar: FfiPolar;
  #instances: Map<number, unknown>;
  types: Map<string | Class, UserType<any>>; // eslint-disable-line @typescript-eslint/no-explicit-any
  #equalityFn: EqualityFn;

  // global data filtering config
  buildQuery: BuildQueryFn;
  execQuery: ExecQueryFn;
  combineQuery: CombineQueryFn;

  /**
   * Shallow clone a host to extend its state for the duration of a particular
   * query without modifying the longer-lived [[`Polar`]] host state.
   *
   * @internal
   */
  static clone(host: Host): Host {
    const clone = new Host(host.#ffiPolar, host.#equalityFn);
    clone.#instances = new Map(host.#instances);
    clone.types = new Map(host.types);
    clone.buildQuery = host.buildQuery;
    clone.execQuery = host.execQuery;
    clone.combineQuery = host.combineQuery;
    return clone;
  }

  /** @internal */
  constructor(ffiPolar: FfiPolar, equalityFn: EqualityFn) {
    this.#ffiPolar = ffiPolar;
    this.#instances = new Map();
    this.#equalityFn = equalityFn;
    this.types = new Map();
    this.buildQuery = () => {
      throw new DataFilteringConfigurationError('buildQuery');
    };
    this.execQuery = () => {
      throw new DataFilteringConfigurationError('execQuery');
    };
    this.combineQuery = () => {
      throw new DataFilteringConfigurationError('combineQuery');
    };
  }

  /**
   * Fetch a JavaScript class from the class cache.
   *
   * @param name Class name to look up.
   *
   * @internal
   */
  private getClass(name: string): Class {
    const typ = this.types.get(name);
    if (typ === undefined) throw new UnregisteredClassError(name);
    return typ.cls;
  }

  /**
   * Get user type for `cls`.
   *
   * @param cls Class or class name.
   */
  getType<Type extends Class>(cls: Type | string): UserType<Type> | undefined {
    return this.types.get(cls);
  }

  /**
   * Return user types that are registered with Host.
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private *distinctUserTypes(): IterableIterator<UserType<any>> {
    for (const [name, typ] of this.types) if (isString(name)) yield typ;
  }

  serializeTypes(): string {
    const polarTypes: { [tag: string]: SerializedFields } = {};
    for (const [tag, userType] of this.types) {
      if (isString(tag)) {
        const fields = userType.fields;
        const fieldTypes: SerializedFields = {};
        for (const [k, v] of fields) {
          if (v instanceof Relation) {
            fieldTypes[k] = v.serialize();
          } else {
            const class_tag = this.getType(v)?.name;
            if (class_tag === undefined)
              throw new UnregisteredClassError(v.name);
            fieldTypes[k] = { Base: { class_tag } };
          }
        }
        polarTypes[tag] = fieldTypes;
      }
    }
    return JSON.stringify(polarTypes);
  }

  /**
   * Store a JavaScript class in the class cache.
   *
   * @param cls Class to cache.
   * @param params Optional parameters.
   *
   * @internal
   */
  cacheClass(cls: Class, params?: ClassParams): string {
    params = params ? params : {};

    // TODO(gw) maybe we only want to support plain objects?
    let fields = params.fields || {};
    if (!(fields instanceof Map)) fields = new Map(Object.entries(fields));

    const { name, buildQuery, execQuery, combineQuery } = params;
    if (!isConstructor(cls)) throw new InvalidConstructorError(cls);
    const clsName: string = name ? name : cls.name;
    const existing = this.types.get(clsName);
    if (existing) {
      throw new DuplicateClassAliasError({
        name: clsName,
        cls,
        existing,
      });
    }

    const userType = new UserType({
      name: clsName,
      cls,
      fields,
      buildQuery: buildQuery || this.buildQuery,
      execQuery: execQuery || this.execQuery,
      combineQuery: combineQuery || this.combineQuery,
      id: this.cacheInstance(cls),
    });
    this.types.set(cls, userType);
    this.types.set(clsName, userType);
    return clsName;
  }

  /**
   * Return cached instances.
   *
   * Only used by the test suite.
   *
   * @internal
   */
  instances(): unknown[] {
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
  getInstance(id: number): unknown {
    if (!this.hasInstance(id)) throw new UnregisteredInstanceError(id);
    return this.#instances.get(id);
  }

  /**
   * Store a JavaScript instance in the instance cache, fetching a new instance
   * ID from the Polar VM if an ID is not provided.
   *
   * @internal
   */
  cacheInstance(instance: unknown, id?: number): number {
    let instanceId = id;
    if (instanceId === undefined) {
      instanceId = this.#ffiPolar.newId();
    }
    this.#instances.set(instanceId, instance);
    return instanceId;
  }

  /**
   * Register the MROs of all registered classes.
   */
  registerMros() {
    // Get MRO of all registered classes
    // NOTE: not ideal that the MRO gets updated each time loadStr is
    // called, but since we are planning to move to only calling load once
    // with the include feature, I think it's okay for now.
    for (const typ of this.distinctUserTypes()) {
      // Get MRO for type.
      const mro = ancestors(typ.cls)
        .map(c => this.getType(c as Class)?.id)
        .filter(id => id !== undefined);

      // Register with core.
      this.#ffiPolar.registerMro(typ.name, mro);
    }
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
    if (!isObj(instance)) return false;
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
   * Check if the left class is a subclass of the right class.
   *
   * @internal
   */
  async isSubclass(left: string, right: string): Promise<boolean> {
    const leftCls = this.getClass(left);
    const rightCls = this.getClass(right);
    const mro = ancestors(leftCls);
    return mro.includes(rightCls);
  }

  /**
   * Check if the given instance is an instance of a particular class.
   *
   * @internal
   */
  async isa(polarInstance: PolarTerm, name: string): Promise<boolean> {
    const instance = await this.toJs(polarInstance);
    const cls = this.getClass(name);
    return instance instanceof cls || (instance as any)?.constructor === cls; // eslint-disable-line @typescript-eslint/no-explicit-any
  }

  /**
   * Check if a sequence of field accesses on the given class is an
   * instance of another class.
   *
   * @internal
   */
  async isaWithPath(
    baseTag: string,
    path: PolarTerm[],
    classTag: string
  ): Promise<boolean> {
    let tag = baseTag;
    for (const fld of path) {
      const field = await this.toJs(fld);
      if (!isString(field)) throw new Error(`Not a field name: ${field}`);
      const userType = this.types.get(tag);
      if (userType === undefined) return false;

      let fieldType = userType.fields.get(field);
      if (fieldType === undefined) return false;

      if (fieldType instanceof Relation) {
        switch (fieldType.kind) {
          case 'one': {
            const otherCls = this.getType(fieldType.otherType)?.cls;
            if (otherCls === undefined)
              throw new UnregisteredClassError(fieldType.otherType);
            fieldType = otherCls;
            break;
          }
          case 'many':
            fieldType = Array;
            break;
        }
      }

      const newBase = this.getType(fieldType);
      if (newBase === undefined) return false;
      tag = newBase.name;
    }
    return classTag === tag;
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
    // NOTE(gj): These are `any` because JS puts no type boundaries on what's
    // comparable. Want to resolve `{} > NaN` to an arbitrary boolean? Go nuts!
    const left = (await this.toJs(leftTerm)) as any; // eslint-disable-line @typescript-eslint/no-explicit-any
    const right = (await this.toJs(rightTerm)) as any; // eslint-disable-line @typescript-eslint/no-explicit-any
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
      default: {
        const _: never = op;
        return _;
      }
    }
  }

  /**
   * Turn a JavaScript value into a Polar term that's ready to be sent to the
   * Polar VM.
   *
   * @internal
   */
  toPolar(v: unknown): PolarTerm {
    switch (true) {
      case typeof v === 'boolean':
        return { value: { Boolean: v as boolean } };
      case Number.isInteger(v):
        return { value: { Number: { Integer: v as number } } };
      case typeof v === 'number':
        if (v === Infinity) {
          v = 'Infinity';
        } else if (v === -Infinity) {
          v = '-Infinity';
        } else if (Number.isNaN(v)) {
          v = 'NaN';
        }
        return { value: { Number: { Float: v as number } } };
      case isString(v):
        return { value: { String: v as string } };
      case Array.isArray(v): {
        const polarTermList = (v as Array<unknown>).map(a => this.toPolar(a));
        return { value: { List: polarTermList } };
      }
      case v instanceof Predicate: {
        const { name, args } = v as Predicate;
        const polarArgs = args.map(a => this.toPolar(a));
        return { value: { Call: { name, args: polarArgs } } };
      }
      case v instanceof Variable:
        return { value: { Variable: (v as Variable).name } };
      case v instanceof Expression: {
        const { operator, args } = v as Expression;
        const polarArgs = args.map(a => this.toPolar(a));
        return { value: { Expression: { operator, args: polarArgs } } };
      }
      case v instanceof Pattern: {
        const { tag, fields } = v as Pattern;
        let dict = this.toPolar(fields).value;
        // TODO(gj): will `dict.Dictionary` ever be undefined?
        if (!isPolarDict(dict)) dict = { Dictionary: { fields: new Map() } };
        if (tag === undefined) return { value: { Pattern: dict } };
        return {
          value: { Pattern: { Instance: { tag, fields: dict.Dictionary } } },
        };
      }
      case v instanceof Dict: {
        const fields = new Map(
          Object.entries(v as Dict).map(([k, v]) => [k, this.toPolar(v)])
        );
        return { value: { Dictionary: { fields } } };
      }
      default: {
        let instanceId: number | undefined = undefined;
        if (isConstructor(v)) instanceId = this.getType(v)?.id;
        const instance_id = this.cacheInstance(v, instanceId);
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
  }

  /**
   * Turn a Polar term from the Polar VM into a JavaScript value.
   *
   * @internal
   */
  async toJs(v: PolarTerm): Promise<unknown> {
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
        this.toJs(v).then(v => [k, v]) as Promise<[string, unknown]>;
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
      const { name, args } = t.Call;
      const jsArgs = await Promise.all(args.map(a => this.toJs(a)));
      return new Predicate(name, jsArgs);
    } else if (isPolarVariable(t)) {
      return new Variable(t.Variable);
    } else if (isPolarExpression(t)) {
      // TODO(gj): Only allow expressions if the flag has been frobbed.
      const { operator, args: argTerms } = t.Expression;
      const args = await Promise.all(argTerms.map(a => this.toJs(a)));
      return new Expression(operator, args);
    } else if (isPolarPattern(t)) {
      if ('Dictionary' in t.Pattern) {
        const fields = (await this.toJs({ value: t.Pattern })) as Dict;
        return new Pattern({ fields });
      } else {
        const {
          tag,
          fields: { fields },
        } = t.Pattern.Instance;
        const dict = await this.toJs({ value: { Dictionary: { fields } } });
        return new Pattern({ tag, fields: dict as Dict });
      }
    } else {
      const _: never = t;
      return _;
    }
  }
}
