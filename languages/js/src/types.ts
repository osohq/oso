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

function isPolarInstance(v: PolarType): v is PolarInstance {
  return (v as PolarInstance).ExternalInstance !== undefined;
}

function isPolarPredicate(v: PolarType): v is PolarPredicate {
  return (v as PolarPredicate).Call !== undefined;
}

function isPolarVariable(v: PolarType): v is PolarVariable {
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

function isPolarValue(v: any): v is PolarValue {
  return isPolarType(v?.value);
}

interface ConstructorKwargs {
  [key: string]: any;
}

type Constructor = (kwargs: ConstructorKwargs) => object;

class Predicate {
  readonly name: string;
  readonly args: unknown[];

  constructor(name: string, args: unknown[]) {
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

function isGenerator(x: any): x is Generator {
  return [x.next, x.return, x.throw].every(f => typeof f === 'function');
}

function isGeneratorFunction(x: any): x is GeneratorFunction {
  if (!x.constructor) return false;
  return (
    x.constructor.name === 'GeneratorFunction' ||
    isGenerator(x.constructor.prototype)
  );
}

interface Result {
  bindings: Map<string, PolarValue>;
}

interface MakeExternal {
  instanceId: bigint;
  tag: string;
  fields: Map<string, PolarValue>;
}

interface ExternalCall {
  callId: bigint;
  instance: PolarValue;
  attribute: string;
  args: PolarValue[];
}

interface ExternalIsSubspecializer {
  instanceId: bigint;
  leftTag: string;
  rightTag: string;
  callId: bigint;
}

interface ExternalIsa {
  instanceId: bigint;
  tag: string;
  callId: bigint;
}

interface ExternalUnify {
  leftId: bigint;
  rightId: bigint;
  callId: bigint;
}

interface Debug {
  message: string;
}

enum QueryEventKind {
  Done,
  Result,
  MakeExternal,
  ExternalCall,
  ExternalIsSubspecializer,
  ExternalIsa,
  ExternalUnify,
  Debug,
}

interface QueryEvent {
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
