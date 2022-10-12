import {
  DataFilteringConfigurationError,
  DuplicateClassAliasError,
  InvalidConstructorError,
  PolarError,
  UnregisteredClassError,
  UnregisteredInstanceError,
  UnexpectedExpressionError,
} from './errors';
import { ancestors, isConstructor, isString, repr } from './helpers';
import type { Polar as FfiPolar } from './polar_wasm_api';
import { Expression } from './Expression';
import { Pattern } from './Pattern';
import { Predicate } from './Predicate';
import { Variable } from './Variable';
import type {
  Class,
  ClassParams,
  HostOpts,
  PolarComparisonOperator,
  PolarTerm,
  NullishOrHasConstructor,
  HostTypes,
  obj,
} from './types';
import { UserType } from './types';
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
import { Relation, Adapter } from './filter';

/**
 * Translator between Polar and JavaScript.
 *
 * @internal
 */
export class Host<Query, Resource> {
  #ffiPolar: FfiPolar;
  #instances: Map<number, unknown>;
  types: HostTypes;

  #opts: HostOpts;

  adapter: Adapter<Query, Resource>;

  /**
   * Shallow clone a host to extend its state for the duration of a particular
   * query without modifying the longer-lived [[`Polar`]] host state.
   *
   * @internal
   */
  static clone<Query, Resource>(
    host: Host<Query, Resource>,
    opts: Partial<HostOpts>
  ): Host<Query, Resource> {
    const options = { ...host.#opts, ...opts };
    const clone = new Host<Query, Resource>(host.#ffiPolar, options);
    clone.#instances = new Map(host.#instances);
    clone.types = new Map(host.types);
    clone.adapter = host.adapter;
    return clone;
  }

  /** @internal */
  constructor(ffiPolar: FfiPolar, opts: HostOpts) {
    this.#ffiPolar = ffiPolar;
    this.#opts = opts;
    this.#instances = new Map();
    this.types = new Map();

    this.adapter = {
      buildQuery: () => {
        throw new DataFilteringConfigurationError();
      },
      executeQuery: () => {
        throw new DataFilteringConfigurationError();
      },
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
    return typ.cls as Class;
  }

  /**
   * Get user type for `cls`.
   *
   * @param cls Class or class name.
   */
  getType<Type extends Class>(cls?: Type | string): UserType<Type> | undefined {
    if (cls === undefined) return undefined;
    return this.types.get(cls);
  }

  /**
   * Return user types that are registered with Host.
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private *distinctUserTypes(): IterableIterator<UserType<any>> {
    for (const [name, typ] of this.types) if (isString(name)) yield typ;
  }

  serializeTypes(): obj {
    const polarTypes: obj = {};
    for (const [tag, userType] of this.types) {
      if (isString(tag)) {
        const fields = userType.fields;
        const fieldTypes: obj = {};
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
    return polarTypes;
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

    const { name } = params;
    if (!isConstructor(cls)) throw new InvalidConstructorError(cls);
    const clsName: string = name ? name : cls.name;
    const existing = this.types.get(clsName);
    if (existing) {
      throw new DuplicateClassAliasError({
        name: clsName,
        cls,
        existing: existing.cls as Class,
      });
    }

    function defaultCheck(instance: NullishOrHasConstructor): boolean {
      return instance instanceof cls || instance?.constructor === cls;
    }

    const userType = new UserType({
      name: clsName,
      cls,
      fields,
      id: this.cacheInstance(cls),
      isaCheck: params.isaCheck || defaultCheck,
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
  registerMros(): void {
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
    let instance = this.getInstance(id) as NullishOrHasConstructor;
    instance = instance instanceof Promise ? await instance : instance; // eslint-disable-line @typescript-eslint/no-unsafe-assignment
    const mro = ancestors(instance?.constructor);
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
  isSubclass(left: string, right: string): boolean {
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

    const userType = this.types.get(name);
    if (userType !== undefined) {
      return userType.isaCheck(instance);
    } else {
      const cls = this.getClass(name);
      const inst = instance as NullishOrHasConstructor;
      return inst instanceof cls || inst?.constructor === cls;
    }
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
      if (!isString(field)) throw new Error(`Not a field name: ${repr(field)}`);
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
    const left = (await this.toJs(leftTerm)) as any; // eslint-disable-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-assignment
    const right = (await this.toJs(rightTerm)) as any; // eslint-disable-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-assignment
    switch (op) {
      case 'Eq':
        return this.#opts.equalityFn(left, right);
      case 'Geq':
        return left >= right;
      case 'Gt':
        return left > right;
      case 'Leq':
        return left <= right;
      case 'Lt':
        return left < right;
      case 'Neq':
        return !this.#opts.equalityFn(left, right);
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
        let classId: number | undefined = undefined;

        // pass a string class repr *for registered types only*, otherwise pass
        // undefined (allow core to differentiate registered or not)
        const v_cast = v as NullishOrHasConstructor;
        let classRepr: string | undefined = undefined;

        if (isConstructor(v)) {
          instanceId = this.getType(v)?.id;
          classId = instanceId;
          classRepr = this.getType(v)?.name;
        } else {
          const v_constructor: Class | undefined = v_cast?.constructor;

          // pass classId for instances of *registered classes* only
          if (v_constructor !== undefined && this.types.has(v_constructor)) {
            classId = this.getType(v_constructor)?.id;
            classRepr = this.getType(v_constructor)?.name;
          }
        }

        // pass classRepr for *registered* classes only, pass undefined
        // otherwise
        if (classRepr !== undefined && !this.types.has(classRepr)) {
          classRepr = undefined;
        }

        const instance_id = this.cacheInstance(v, instanceId);
        return {
          value: {
            ExternalInstance: {
              instance_id,
              constructor: undefined,
              repr: repr(v),
              class_repr: classRepr,
              class_id: classId,
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
      const valueToJs = ([k, v]: [string, PolarTerm]): Promise<
        [string, unknown]
      > => this.toJs(v).then(v => [k, v]);
      const { fields } = t.Dictionary;
      const entries = await Promise.all([...fields.entries()].map(valueToJs));
      return entries.reduce((dict: Dict, [k, v]) => {
        dict[k] = v;
        return dict;
      }, new Dict({}));
    } else if (isPolarInstance(t)) {
      const i = this.getInstance(t.ExternalInstance.instance_id);
      return i instanceof Promise ? await i : i; // eslint-disable-line @typescript-eslint/no-unsafe-return
    } else if (isPolarPredicate(t)) {
      const { name, args } = t.Call;
      const jsArgs = await Promise.all(args.map(a => this.toJs(a)));
      return new Predicate(name, jsArgs);
    } else if (isPolarVariable(t)) {
      return new Variable(t.Variable);
    } else if (isPolarExpression(t)) {
      if (!this.#opts.acceptExpression) throw new UnexpectedExpressionError();

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
