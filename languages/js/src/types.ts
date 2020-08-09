interface PolarStr {
  String: string;
}

export function isPolarStr(v: PolarType): v is PolarStr {
  return (v as PolarStr).String !== undefined;
}

interface PolarNum {
  Number: PolarFloat | PolarInt;
}

export function isPolarNum(v: PolarType): v is PolarNum {
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

export function isPolarBool(v: PolarType): v is PolarBool {
  return (v as PolarBool).Boolean !== undefined;
}

interface PolarList {
  List: PolarValue[];
}

export function isPolarList(v: PolarType): v is PolarList {
  return (v as PolarList).List !== undefined;
}

interface PolarDict {
  Dictionary: {
    fields: {
      [key: string]: PolarValue;
    };
  };
}

export function isPolarDict(v: PolarType): v is PolarDict {
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

export function isPolarInstance(v: PolarType): v is PolarInstance {
  return (v as PolarInstance).ExternalInstance !== undefined;
}

export function isPolarPredicate(v: PolarType): v is PolarPredicate {
  return (v as PolarPredicate).Call !== undefined;
}

export function isPolarVariable(v: PolarType): v is PolarVariable {
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

export interface PolarValue {
  value: PolarType;
}

function isPolarType(v: any): v is PolarType {
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

export function isPolarValue(v: any): v is PolarValue {
  return isPolarType(v?.value);
}

export interface ConstructorKwargs {
  [key: string]: any;
}

export type Constructor = (kwargs: ConstructorKwargs) => object;

export function isGenerator(x: any): x is Generator {
  return [x.next, x.return, x.throw].every(f => typeof f === 'function');
}

export function isGeneratorFunction(x: any): x is GeneratorFunction {
  if (!x.constructor) return false;
  return (
    x.constructor.name === 'GeneratorFunction' ||
    isGenerator(x.constructor.prototype)
  );
}

export interface Result {
  bindings: Map<string, PolarValue>;
}

export interface MakeExternal {
  instanceId: bigint;
  tag: string;
  fields: Map<string, PolarValue>;
}

export interface ExternalCall {
  callId: bigint;
  instance: PolarValue;
  attribute: string;
  args: PolarValue[];
}

export interface ExternalIsSubspecializer {
  instanceId: bigint;
  leftTag: string;
  rightTag: string;
  callId: bigint;
}

export interface ExternalIsa {
  instanceId: bigint;
  tag: string;
  callId: bigint;
}

export interface ExternalUnify {
  leftId: bigint;
  rightId: bigint;
  callId: bigint;
}

export interface Debug {
  message: string;
}

export enum QueryEventKind {
  Done,
  Result,
  MakeExternal,
  ExternalCall,
  ExternalIsSubspecializer,
  ExternalIsa,
  ExternalUnify,
  Debug,
}

export interface QueryEvent {
  kind: QueryEventKind;
  data?:
    | Result
    | MakeExternal
    | ExternalCall
    | ExternalIsSubspecializer
    | ExternalIsa
    | ExternalUnify
    | Debug;
}

export type QueryResult = Generator<Map<string, any>, null, never>;
