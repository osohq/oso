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
  List: PolarTerm[];
}

export function isPolarList(v: PolarType): v is PolarList {
  return (v as PolarList).List !== undefined;
}

interface PolarDict {
  Dictionary: {
    fields: Map<string, PolarTerm> | { [key: string]: PolarTerm };
  };
}

export function isPolarDict(v: PolarType): v is PolarDict {
  return (v as PolarDict).Dictionary !== undefined;
}

interface PolarPredicate {
  Call: {
    name: string;
    args: PolarTerm[];
  };
}

interface PolarVariable {
  Variable: string;
}

interface PolarInstance {
  ExternalInstance: {
    instance_id: number;
    repr: string;
    constructor?: PolarTerm;
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

export interface PolarTerm {
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

export function isPolarTerm(v: any): v is PolarTerm {
  return isPolarType(v?.value);
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

export type QueryResult = Generator<Map<string, any>, void, never>;

export type obj = { [key: string]: any };

export type EqualityFn = (x: any, y: any) => boolean;

export interface Options {
  equalityFn?: EqualityFn;
}
