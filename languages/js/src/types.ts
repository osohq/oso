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
 *
 * @internal
 */
interface PolarFloat {
  Float: number;
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
    fields: Map<string, PolarTerm> | { [key: string]: PolarTerm };
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
    constructor?: PolarTerm;
  };
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
 * Union of Polar value types.
 *
 * @internal
 */
type PolarValue =
  | PolarStr
  | PolarNum
  | PolarBool
  | PolarList
  | PolarDict
  | PolarPredicate
  | PolarVariable
  | PolarInstance;

/**
 * Union of Polar value types.
 *
 * @internal
 */
export interface PolarTerm {
  value: PolarValue;
}

function isPolarValue(v: any): v is PolarValue {
  return (
    isPolarStr(v) ||
    isPolarNum(v) ||
    isPolarBool(v) ||
    isPolarList(v) ||
    isPolarDict(v) ||
    isPolarPredicate(v) ||
    isPolarVariable(v) ||
    isPolarInstance(v)
  );
}

export function isPolarTerm(v: any): v is PolarTerm {
  return isPolarValue(v?.value);
}

export type Class<T extends {} = {}> = new (...args: any[]) => T;

export interface Result {
  bindings: Map<string, PolarTerm>;
}

export interface MakeExternal {
  instanceId: number;
  tag: string;
  fields: PolarTerm[];
}

export interface ExternalCall {
  callId: number;
  instance: PolarTerm;
  attribute: string;
  args?: PolarTerm[];
}

export interface ExternalIsSubspecializer {
  instanceId: number;
  leftTag: string;
  rightTag: string;
  callId: number;
}

export interface ExternalIsa {
  instance: PolarTerm;
  tag: string;
  callId: number;
}

export interface ExternalUnify {
  leftId: number;
  rightId: number;
  callId: number;
}

export interface Debug {
  message: string;
}

export enum QueryEventKind {
  Debug,
  Done,
  ExternalCall,
  ExternalIsa,
  ExternalIsSubspecializer,
  ExternalUnify,
  MakeExternal,
  Result,
}

export interface QueryEvent {
  kind: QueryEventKind;
  data?:
    | Debug
    | ExternalCall
    | ExternalIsa
    | ExternalIsSubspecializer
    | ExternalUnify
    | MakeExternal
    | Result;
}

export type QueryResult = AsyncGenerator<
  Map<string, any>,
  void,
  undefined | void
>;

export type obj = { [key: string]: any };

export type EqualityFn = (x: any, y: any) => boolean;

export interface Options {
  equalityFn?: EqualityFn;
}

export function isIterableIterator(x: any): boolean {
  return typeof x?.next === 'function' && Symbol.iterator in Object(x);
}

export function isAsyncIterator(x: any): boolean {
  return Symbol.asyncIterator in Object(x);
}
