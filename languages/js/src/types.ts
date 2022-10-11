import type { Relation } from './filter';
import { isObj } from './helpers';

/**
 * Polar string type.
 *
 * @internal
 */
interface PolarStr {
  String: string;
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar string.
 *
 * @internal
 */
export function isPolarStr(v: PolarValue): v is PolarStr {
  return (v as PolarStr).String !== undefined;
}

/**
 * Type guard to test if a string received from across the WebAssembly
 * boundary is a PolarComparisonOperator.
 *
 * @internal
 */
export function isPolarComparisonOperator(
  s: string
): s is PolarComparisonOperator {
  return s in comparisonOperators;
}

/**
 * Polar numeric type.
 *
 * @internal
 */
interface PolarNum {
  Number: PolarFloat | PolarInt;
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar numeric.
 *
 * @internal
 */
export function isPolarNum(v: PolarValue): v is PolarNum {
  return (v as PolarNum).Number !== undefined;
}

/**
 * Polar floating point type.
 * The string variant is to support ±∞ and NaN.
 *
 * @internal
 */
interface PolarFloat {
  Float: number | string;
}

/**
 * Polar integer type.
 *
 * @internal
 */
interface PolarInt {
  Integer: number;
}

/**
 * Polar boolean type.
 *
 * @internal
 */
interface PolarBool {
  Boolean: boolean;
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar boolean.
 *
 * @internal
 */
export function isPolarBool(v: PolarValue): v is PolarBool {
  return (v as PolarBool).Boolean !== undefined;
}

/**
 * Polar list type.
 *
 * @internal
 */
interface PolarList {
  List: PolarTerm[];
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar list.
 *
 * @internal
 */
export function isPolarList(v: PolarValue): v is PolarList {
  return (v as PolarList).List !== undefined;
}

/**
 * Polar dictionary type.
 *
 * @internal
 */
interface PolarDict {
  Dictionary: {
    fields: Map<string, PolarTerm>;
  };
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar dictionary.
 *
 * @internal
 */
export function isPolarDict(v: PolarValue): v is PolarDict {
  return (v as PolarDict).Dictionary !== undefined;
}

/**
 * Polar predicate type.
 *
 * @internal
 */
interface PolarPredicate {
  Call: {
    name: string;
    args: PolarTerm[];
  };
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar predicate.
 *
 * @internal
 */
export function isPolarPredicate(v: PolarValue): v is PolarPredicate {
  return (v as PolarPredicate).Call !== undefined;
}

/**
 * Polar variable type.
 *
 * @internal
 */
interface PolarVariable {
  Variable: string;
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar variable.
 *
 * @internal
 */
export function isPolarVariable(v: PolarValue): v is PolarVariable {
  return (v as PolarVariable).Variable !== undefined;
}

/**
 * Polar application instance type.
 *
 * @internal
 */
interface PolarInstance {
  ExternalInstance: {
    instance_id: number;
    repr: string;
    class_repr?: string;
    class_id?: number;
    constructor?: PolarTerm;
  };
}

/**
 * Polar expression type.
 *
 * @internal
 */
interface PolarExpression {
  Expression: {
    args: PolarTerm[];
    operator: PolarOperator;
  };
}

/**
 * Polar instance (tagged dict) pattern variant.
 *
 * @internal
 */
interface PolarInstancePattern {
  Instance: {
    tag?: string;
    fields: { fields: Map<string, PolarTerm> };
  };
}

/**
 * Polar (untagged) dict pattern variant.
 *
 * @internal
 */
export type PolarDictPattern = PolarDict;

/**
 * Polar pattern type.
 *
 * @internal
 */
interface PolarPattern {
  Pattern: PolarDictPattern | PolarInstancePattern;
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar application instance.
 *
 * @internal
 */
export function isPolarInstance(v: PolarValue): v is PolarInstance {
  return (v as PolarInstance).ExternalInstance !== undefined;
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar expression.
 *
 * @internal
 */
export function isPolarExpression(v: PolarValue): v is PolarExpression {
  return (v as PolarExpression).Expression !== undefined;
}

/**
 * Type guard to test if a Polar value received from across the WebAssembly
 * boundary is a Polar pattern.
 *
 * @internal
 */
export function isPolarPattern(v: PolarValue): v is PolarPattern {
  return (v as PolarPattern).Pattern !== undefined;
}

/**
 * Union of Polar value types.
 *
 * @internal
 */
export type PolarValue =
  | PolarStr
  | PolarNum
  | PolarBool
  | PolarList
  | PolarDict
  | PolarPredicate
  | PolarVariable
  | PolarInstance
  | PolarExpression
  | PolarPattern;

/**
 * Union of Polar value types.
 *
 * @internal
 */
export interface PolarTerm {
  value: PolarValue;
}

/**
 * Type guard to test if a JSON payload received from across the WebAssembly
 * boundary contains a valid Polar value.
 *
 * @internal
 */
function isPolarValue(x: unknown): x is PolarValue {
  if (!isObj(x)) return false;
  const v = x as unknown as PolarValue;
  return (
    isPolarStr(v) ||
    isPolarNum(v) ||
    isPolarBool(v) ||
    isPolarList(v) ||
    isPolarDict(v) ||
    isPolarPredicate(v) ||
    isPolarVariable(v) ||
    isPolarInstance(v) ||
    isPolarExpression(v) ||
    isPolarPattern(v)
  );
}

/**
 * Type guard to test if a JSON payload received from across the WebAssembly
 * boundary contains a valid Polar term.
 *
 * @internal
 */
export function isPolarTerm(v: unknown): v is PolarTerm {
  if (!isObj(v)) return false;
  return isPolarValue(v.value);
}

/**
 * A constructable (via the `new` keyword) application class.
 *
 * @internal
 */
export type Class<T = unknown> = new (...args: any[]) => T; // eslint-disable-line @typescript-eslint/no-explicit-any

/**
 * The `Result` [[`QueryEvent`]] represents a single result from a query
 * containing any variables bound during the evaluation of the query.
 *
 * @internal
 */
export interface Result {
  bindings: Map<string, PolarTerm>;
}

/**
 * The `MakeExternal` [[`QueryEvent`]] is how Polar constructs application
 * instances during the evaluation of a query.
 *
 * @internal
 */
export interface MakeExternal {
  instanceId: number;
  tag: string;
  fields: PolarTerm[];
}

/**
 * The `NextExternal` [[`QueryEvent`]] is how Polar iterates
 * through iterable host values.
 *
 * @internal
 */
export interface NextExternal {
  callId: number;
  iterable: PolarTerm;
}

/**
 * The `ExternalCall` [[`QueryEvent`]] is how Polar invokes JavaScript
 * functions registered as constants, methods on built-in types, and methods on
 * registered application classes during the evaluation of a query.
 *
 * @internal
 */
export interface ExternalCall {
  callId: number;
  instance: PolarTerm;
  attribute: string;
  args?: PolarTerm[];
}

/**
 * The `ExternalIsSubspecializer` [[`QueryEvent`]] is how Polar determines
 * which of two classes is more specific with respect to a given instance.
 *
 * @internal
 */
export interface ExternalIsSubspecializer {
  instanceId: number;
  leftTag: string;
  rightTag: string;
  callId: number;
}

/**
 * The `ExternalIsa` [[`QueryEvent`]] is how Polar determines whether a given
 * value is an instance of a particular class.
 *
 * @internal
 */
export interface ExternalIsa {
  instance: PolarTerm;
  tag: string;
  callId: number;
}

/**
 * The `ExternalIsaWithPath` [[`QueryEvent`]] is how Polar determines whether a given
 * sequence of field accesses on a value is an instance of a particular class.
 *
 * @internal
 */
export interface ExternalIsaWithPath {
  baseTag: string;
  path: PolarTerm[];
  classTag: string;
  callId: number;
}

/**
 * The `ExternalIsSubclass` [[`QueryEvent`]] is how Polar determines whether a given
 * class is a subclass of a particular class.
 *
 * @internal
 */
export interface ExternalIsSubclass {
  leftTag: string;
  rightTag: string;
  callId: number;
}

/**
 * Polar comparison operators.
 *
 * Currently, these are the only operators supported for external operations.
 *
 * @internal
 */
const comparisonOperators = {
  Eq: 'Eq',
  Geq: 'Geq',
  Gt: 'Gt',
  Leq: 'Leq',
  Lt: 'Lt',
  Neq: 'Neq',
} as const;
export type PolarComparisonOperator = keyof typeof comparisonOperators;

/**
 * Polar operators.
 *
 * @internal
 */
const operators = {
  Add: 'Add',
  And: 'And',
  Assign: 'Assign',
  Cut: 'Cut',
  Debug: 'Debug',
  Div: 'Div',
  Dot: 'Dot',
  ForAll: 'ForAll',
  In: 'In',
  Isa: 'Isa',
  Mod: 'Mod',
  Mul: 'Mul',
  New: 'New',
  Not: 'Not',
  Or: 'Or',
  Print: 'Print',
  Rem: 'Rem',
  Sub: 'Sub',
  Unify: 'Unify',
  ...comparisonOperators,
} as const;
export type PolarOperator = keyof typeof operators;

/**
 * The `ExternalOp` [[`QueryEvent`]] is how Polar evaluates an operation
 * involving one or more application instances.
 *
 * @internal
 */
export interface ExternalOp {
  args: PolarTerm[];
  callId: number;
  operator: PolarComparisonOperator;
}

/**
 * The `Debug` [[`QueryEvent`]] is how Polar relays debugging messages to
 * JavaScript from the internal debugger attached to the Polar VM.
 *
 * @internal
 */
export interface Debug {
  message: string;
}

/**
 * Union of all [[`QueryEvent`]] types.
 *
 * @internal
 */
export enum QueryEventKind {
  Debug,
  Done,
  ExternalCall,
  ExternalIsa,
  ExternalIsaWithPath,
  ExternalIsSubspecializer,
  ExternalIsSubclass,
  ExternalOp,
  MakeExternal,
  NextExternal,
  Result,
}

/**
 * An event from the Polar VM.
 *
 * @internal
 */
export interface QueryEvent {
  kind: QueryEventKind;
  data?:
    | Debug
    | ExternalCall
    | ExternalIsa
    | ExternalIsaWithPath
    | ExternalIsSubspecializer
    | ExternalIsSubclass
    | ExternalOp
    | MakeExternal
    | NextExternal
    | Result;
}

/**
 * An `AsyncGenerator` over query results.
 *
 * Each result is a `Map` of variables bound during the computation of that
 * result.
 *
 * If you don't need access to the bindings and only wish to know whether a
 * query succeeded or failed, you may check the `done` property of the yielded
 * value (and then optionally "complete" the generator by calling and awaiting its
 * `return()` method). If `done` is `true`, the query failed. If `done` is
 * `false`, the query yielded at least one result and therefore succeeded.
 */
export type QueryResult = AsyncGenerator<
  Map<string, unknown>,
  void,
  undefined | void
>;

/**
 * Optional configuration for [[`Polar.query`]] and [[`Polar.queryRule`]].
 */
export type QueryOpts = {
  /**
   * Opt-in flag indicating whether [[`Host`]] can receive [[`Expression`]]s
   * from core for duration of query. When `false`, [[`Host`]] errors on
   * receiving [[`Expression`]] from core. Main use is for indicating whether
   * the consumer of the result bindings is prepared to handle constraints
   * ([[`Expression`]]s) received from core for data filtering purposes.
   */
  acceptExpression?: boolean;
  /**
   * Bind keys to values in VM for duration of query.
   */
  bindings?: Map<string, unknown>;
};

/**
 * Required configuration for [[`Host`]].
 *
 * @internal
 */
export type HostOpts = {
  /**
   * Opt-in flag indicating whether [[`Host`]] can receive [[`Expression`]]s
   * from core. When `false`, [[`Host`]] errors on receiving [[`Expression`]]
   * from core. Main use is for indicating whether the consumer of the result
   * bindings is prepared to handle constraints ([[`Expression`]]s) received
   * from core for data filtering purposes.
   */
  acceptExpression: boolean;
  equalityFn: EqualityFn;
};

/**
 * An object with string keys.
 *
 * @hidden
 */
export type obj<T = unknown> = { [key: string]: T };

/**
 * A function that compares two values and returns `true` if they are equal and
 * `false` otherwise.
 *
 * A custom `EqualityFn` may be passed in the [[`Options`]] provided to the
 * [[`Oso.constructor`]] in order to override the default equality function,
 * which uses `isEqual` from the `lodash.isequal` package.
 */
export type EqualityFn = (x: unknown, y: unknown) => boolean;

export type CustomError = new (...args: any[]) => Error; // eslint-disable-line @typescript-eslint/no-explicit-any

/**
 * Optional configuration for the [[`Oso.constructor`]].
 */
export interface Options {
  equalityFn?: EqualityFn;
  /**
   * Optionally override the "not found" error class thrown by `authorize`.
   * Defaults to {@link NotFoundError}.
   */
  notFoundError?: CustomError;
  /**
   * Optionally override the "forbidden" error class thrown by the `authorize*`
   * methods. Defaults to {@link ForbiddenError}.
   */
  forbiddenError?: CustomError;
  /**
   * The action used by the `authorize` method to determine whether an
   * authorization failure should raise a `NotFoundError` or a `ForbiddenError`.
   */
  readAction?: unknown;
}

/**
 * Type guard to test if a value conforms to both the iterable and iterator
 * protocols. This is basically a slightly relaxed check for whether the value
 * is a `Generator`.
 *
 * @internal
 */
export function isIterableIterator(x: unknown): x is IterableIterator<unknown> {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  return typeof x?.next === 'function' && isIterable(x);
}

/**
 * Type guard to test if a value is an `Iterable`.
 *
 * @internal
 */
export function isIterable(x: unknown): x is Iterable<unknown> {
  try {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    return Symbol.iterator in x;
  } catch (e) {
    if (e instanceof TypeError) return false;
    throw e;
  }
}

/**
 * Type guard to test if a value is an `AsyncIterable`.
 *
 * @internal
 */
export function isAsyncIterable(x: unknown): x is AsyncIterable<unknown> {
  try {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    return Symbol.asyncIterator in x;
  } catch (e) {
    if (e instanceof TypeError) return false;
    throw e;
  }
}

/**
 * JS analogue of Polar's Dictionary type.
 *
 * Polar dictionaries allow field access via the dot operator, which mirrors
 * the way JS objects behave. However, if we translate Polar dictionaries into
 * JS objects, we lose the ability to distinguish between dictionaries and
 * instances, since all JS instances are objects. By subclassing `Object`, we
 * can use `instanceof` to determine if a JS value should be serialized as a
 * Polar dictionary or external instance.
 *
 * @internal
 */
export class Dict extends Object {
  [index: string]: unknown;

  // eslint-disable-next-line @typescript-eslint/ban-types
  constructor(obj?: Object) {
    super();
    if (obj) {
      Object.assign(this, obj);
    }
  }
}

export type IsaCheck = (instance: any) => boolean; // eslint-disable-line @typescript-eslint/no-explicit-any

/**
 * Optional parameters for [[`Polar.registerClass`]] and [[`Host.cacheClass`]].
 */
export interface ClassParams {
  /**
   * Explicit name to use for the class in Polar. Defaults to the class's
   * `name` property.
   */
  name?: string;

  /**
   * A Map or object with string keys containing types for fields. Used for
   * data filtering.
   */
  fields?: obj<Class | Relation> | Map<string, Class | Relation>;

  isaCheck?: IsaCheck;
}

/**
 * Parameters for [[`UserType`]].
 */
export interface UserTypeParams<Type extends Class> {
  /**
   * Class registered as a user type.
   */
  cls: Type;
  /**
   * Explicit name to use for the class in Polar.
   */
  name: string;
  /**
   * A Map with string keys containing types for fields. Used for data
   * filtering.
   */
  fields: Map<string, Class | Relation>;
  /**
   * Polar instance ID for the registered class.
   *
   * @internal
   */
  id: number;

  isaCheck: IsaCheck;
}

/**
 * Utility type to represent a JS value that either does or does not have a
 * constructor property.
 *
 * NOTE(gj): I *think* `null` & `undefined` are the only JS values w/o a
 * `constructor` property (e.g., `(1).constructor` returns `[Function:
 * Number]`), but I'm not 100% sure of that.
 */
export type NullishOrHasConstructor = { constructor: Class } | null | undefined;

export type HostTypes = Map<string | Class, UserType<any>>; // eslint-disable-line @typescript-eslint/no-explicit-any

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class UserType<Type extends Class<T>, T = any> {
  name: string;
  cls: Type;
  id: number;
  fields: Map<string, Class | Relation>;
  isaCheck: IsaCheck;

  constructor({ name, cls, id, fields, isaCheck }: UserTypeParams<Type>) {
    this.name = name;
    this.cls = cls;
    this.fields = fields;
    this.id = id;
    this.isaCheck = isaCheck;
  }
}
