# pyre-strict
from dataclasses import dataclass
import typing
import serde_types as st

@dataclass(frozen=True)
class Call:
    name: str
    args: typing.Sequence["Value"]
    kwargs: typing.Optional[typing.Dict[str, "Value"]]


@dataclass(frozen=True)
class Dictionary:
    fields: typing.Dict[str, "Value"]


class ErrorKind:
    pass


@dataclass(frozen=True)
class ErrorKind__Parse(ErrorKind):
    value: "ParseError"


@dataclass(frozen=True)
class ErrorKind__Runtime(ErrorKind):
    value: "RuntimeError"


@dataclass(frozen=True)
class ErrorKind__Operational(ErrorKind):
    value: "OperationalError"


@dataclass(frozen=True)
class ErrorKind__Parameter(ErrorKind):
    value: "ParameterError"

ErrorKind.VARIANTS_MAP = {
    "Parse": ErrorKind__Parse,
    "Runtime": ErrorKind__Runtime,
    "Operational": ErrorKind__Operational,
    "Parameter": ErrorKind__Parameter,
}


@dataclass(frozen=True)
class ExternalInstance:
    instance_id: st.uint64
    constructor: typing.Optional["Value"]
    repr: typing.Optional[str]


@dataclass(frozen=True)
class FormattedPolarError:
    kind: "ErrorKind"
    formatted: str


@dataclass(frozen=True)
class InstanceLiteral:
    tag: str
    fields: "Dictionary"


class Node:
    pass


@dataclass(frozen=True)
class Node__Rule(Node):
    value: "Rule"


@dataclass(frozen=True)
class Node__Term(Node):
    value: "Value"

Node.VARIANTS_MAP = {
    "Rule": Node__Rule,
    "Term": Node__Term,
}


class Numeric:
    pass


@dataclass(frozen=True)
class Numeric__Integer(Numeric):
    value: st.int64


@dataclass(frozen=True)
class Numeric__Float(Numeric):
    value: st.float64

Numeric.VARIANTS_MAP = {
    "Integer": Numeric__Integer,
    "Float": Numeric__Float,
}


@dataclass(frozen=True)
class Operation:
    operator: "Operator"
    args: typing.Sequence["Value"]


class OperationalError:
    pass


@dataclass(frozen=True)
class OperationalError__Unimplemented(OperationalError):
    value: str


@dataclass(frozen=True)
class OperationalError__Unknown(OperationalError):
    pass


@dataclass(frozen=True)
class OperationalError__InvalidState(OperationalError):
    value: str

OperationalError.VARIANTS_MAP = {
    "Unimplemented": OperationalError__Unimplemented,
    "Unknown": OperationalError__Unknown,
    "InvalidState": OperationalError__InvalidState,
}


class Operator:
    pass


@dataclass(frozen=True)
class Operator__Debug(Operator):
    pass


@dataclass(frozen=True)
class Operator__Print(Operator):
    pass


@dataclass(frozen=True)
class Operator__Cut(Operator):
    pass


@dataclass(frozen=True)
class Operator__In(Operator):
    pass


@dataclass(frozen=True)
class Operator__Isa(Operator):
    pass


@dataclass(frozen=True)
class Operator__New(Operator):
    pass


@dataclass(frozen=True)
class Operator__Dot(Operator):
    pass


@dataclass(frozen=True)
class Operator__Not(Operator):
    pass


@dataclass(frozen=True)
class Operator__Mul(Operator):
    pass


@dataclass(frozen=True)
class Operator__Div(Operator):
    pass


@dataclass(frozen=True)
class Operator__Mod(Operator):
    pass


@dataclass(frozen=True)
class Operator__Rem(Operator):
    pass


@dataclass(frozen=True)
class Operator__Add(Operator):
    pass


@dataclass(frozen=True)
class Operator__Sub(Operator):
    pass


@dataclass(frozen=True)
class Operator__Eq(Operator):
    pass


@dataclass(frozen=True)
class Operator__Geq(Operator):
    pass


@dataclass(frozen=True)
class Operator__Leq(Operator):
    pass


@dataclass(frozen=True)
class Operator__Neq(Operator):
    pass


@dataclass(frozen=True)
class Operator__Gt(Operator):
    pass


@dataclass(frozen=True)
class Operator__Lt(Operator):
    pass


@dataclass(frozen=True)
class Operator__Unify(Operator):
    pass


@dataclass(frozen=True)
class Operator__Or(Operator):
    pass


@dataclass(frozen=True)
class Operator__And(Operator):
    pass


@dataclass(frozen=True)
class Operator__ForAll(Operator):
    pass


@dataclass(frozen=True)
class Operator__Assign(Operator):
    pass

Operator.VARIANTS_MAP = {
    "Debug": Operator__Debug,
    "Print": Operator__Print,
    "Cut": Operator__Cut,
    "In": Operator__In,
    "Isa": Operator__Isa,
    "New": Operator__New,
    "Dot": Operator__Dot,
    "Not": Operator__Not,
    "Mul": Operator__Mul,
    "Div": Operator__Div,
    "Mod": Operator__Mod,
    "Rem": Operator__Rem,
    "Add": Operator__Add,
    "Sub": Operator__Sub,
    "Eq": Operator__Eq,
    "Geq": Operator__Geq,
    "Leq": Operator__Leq,
    "Neq": Operator__Neq,
    "Gt": Operator__Gt,
    "Lt": Operator__Lt,
    "Unify": Operator__Unify,
    "Or": Operator__Or,
    "And": Operator__And,
    "ForAll": Operator__ForAll,
    "Assign": Operator__Assign,
}


@dataclass(frozen=True)
class Parameter:
    parameter: "Value"
    specializer: typing.Optional["Value"]


@dataclass(frozen=True)
class ParameterError:
    value: str


class ParseError:
    pass


@dataclass(frozen=True)
class ParseError__IntegerOverflow(ParseError):
    token: str
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__InvalidTokenCharacter(ParseError):
    token: str
    c: st.char
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__InvalidToken(ParseError):
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__UnrecognizedEOF(ParseError):
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__UnrecognizedToken(ParseError):
    token: str
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__ExtraToken(ParseError):
    token: str
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__ReservedWord(ParseError):
    token: str
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__InvalidFloat(ParseError):
    token: str
    loc: st.uint64


@dataclass(frozen=True)
class ParseError__WrongValueType(ParseError):
    loc: st.uint64
    term: "Value"
    expected: str

ParseError.VARIANTS_MAP = {
    "IntegerOverflow": ParseError__IntegerOverflow,
    "InvalidTokenCharacter": ParseError__InvalidTokenCharacter,
    "InvalidToken": ParseError__InvalidToken,
    "UnrecognizedEOF": ParseError__UnrecognizedEOF,
    "UnrecognizedToken": ParseError__UnrecognizedToken,
    "ExtraToken": ParseError__ExtraToken,
    "ReservedWord": ParseError__ReservedWord,
    "InvalidFloat": ParseError__InvalidFloat,
    "WrongValueType": ParseError__WrongValueType,
}


@dataclass(frozen=True)
class Partial:
    constraints: typing.Sequence["Operation"]
    variable: str


class Pattern:
    pass


@dataclass(frozen=True)
class Pattern__Dictionary(Pattern):
    value: "Dictionary"


@dataclass(frozen=True)
class Pattern__Instance(Pattern):
    value: "InstanceLiteral"

Pattern.VARIANTS_MAP = {
    "Dictionary": Pattern__Dictionary,
    "Instance": Pattern__Instance,
}


class QueryEvent:
    pass


@dataclass(frozen=True)
class QueryEvent__None(QueryEvent):
    pass


@dataclass(frozen=True)
class QueryEvent__Done(QueryEvent):
    result: st.bool


@dataclass(frozen=True)
class QueryEvent__Debug(QueryEvent):
    message: str


@dataclass(frozen=True)
class QueryEvent__MakeExternal(QueryEvent):
    instance_id: st.uint64
    constructor: "Value"


@dataclass(frozen=True)
class QueryEvent__ExternalCall(QueryEvent):
    call_id: st.uint64
    instance: "Value"
    attribute: str
    args: typing.Optional[typing.Sequence["Value"]]
    kwargs: typing.Optional[typing.Dict[str, "Value"]]


@dataclass(frozen=True)
class QueryEvent__ExternalIsa(QueryEvent):
    call_id: st.uint64
    instance: "Value"
    class_tag: str


@dataclass(frozen=True)
class QueryEvent__ExternalIsSubSpecializer(QueryEvent):
    call_id: st.uint64
    instance_id: st.uint64
    left_class_tag: str
    right_class_tag: str


@dataclass(frozen=True)
class QueryEvent__ExternalIsSubclass(QueryEvent):
    call_id: st.uint64
    left_class_tag: str
    right_class_tag: str


@dataclass(frozen=True)
class QueryEvent__ExternalUnify(QueryEvent):
    call_id: st.uint64
    left_instance_id: st.uint64
    right_instance_id: st.uint64


@dataclass(frozen=True)
class QueryEvent__Result(QueryEvent):
    bindings: typing.Dict[str, "Value"]
    trace: typing.Optional["TraceResult"]


@dataclass(frozen=True)
class QueryEvent__ExternalOp(QueryEvent):
    call_id: st.uint64
    operator: "Operator"
    args: typing.Sequence["Value"]


@dataclass(frozen=True)
class QueryEvent__NextExternal(QueryEvent):
    call_id: st.uint64
    iterable: "Value"

QueryEvent.VARIANTS_MAP = {
    "None": QueryEvent__None,
    "Done": QueryEvent__Done,
    "Debug": QueryEvent__Debug,
    "MakeExternal": QueryEvent__MakeExternal,
    "ExternalCall": QueryEvent__ExternalCall,
    "ExternalIsa": QueryEvent__ExternalIsa,
    "ExternalIsSubSpecializer": QueryEvent__ExternalIsSubSpecializer,
    "ExternalIsSubclass": QueryEvent__ExternalIsSubclass,
    "ExternalUnify": QueryEvent__ExternalUnify,
    "Result": QueryEvent__Result,
    "ExternalOp": QueryEvent__ExternalOp,
    "NextExternal": QueryEvent__NextExternal,
}


@dataclass(frozen=True)
class Rule:
    name: str
    params: typing.Sequence["Parameter"]
    body: "Value"


class RuntimeError:
    pass


@dataclass(frozen=True)
class RuntimeError__ArithmeticError(RuntimeError):
    msg: str


@dataclass(frozen=True)
class RuntimeError__Serialization(RuntimeError):
    msg: str


@dataclass(frozen=True)
class RuntimeError__Unsupported(RuntimeError):
    msg: str


@dataclass(frozen=True)
class RuntimeError__TypeError(RuntimeError):
    msg: str
    stack_trace: typing.Optional[str]


@dataclass(frozen=True)
class RuntimeError__UnboundVariable(RuntimeError):
    sym: str


@dataclass(frozen=True)
class RuntimeError__StackOverflow(RuntimeError):
    msg: str


@dataclass(frozen=True)
class RuntimeError__QueryTimeout(RuntimeError):
    msg: str


@dataclass(frozen=True)
class RuntimeError__Application(RuntimeError):
    msg: str
    stack_trace: typing.Optional[str]


@dataclass(frozen=True)
class RuntimeError__FileLoading(RuntimeError):
    msg: str

RuntimeError.VARIANTS_MAP = {
    "ArithmeticError": RuntimeError__ArithmeticError,
    "Serialization": RuntimeError__Serialization,
    "Unsupported": RuntimeError__Unsupported,
    "TypeError": RuntimeError__TypeError,
    "UnboundVariable": RuntimeError__UnboundVariable,
    "StackOverflow": RuntimeError__StackOverflow,
    "QueryTimeout": RuntimeError__QueryTimeout,
    "Application": RuntimeError__Application,
    "FileLoading": RuntimeError__FileLoading,
}


@dataclass(frozen=True)
class Trace:
    node: "Node"
    children: typing.Sequence["Trace"]


@dataclass(frozen=True)
class TraceResult:
    trace: "Trace"
    formatted: str


class Value:
    pass


@dataclass(frozen=True)
class Value__Number(Value):
    value: "Numeric"


@dataclass(frozen=True)
class Value__String(Value):
    value: str


@dataclass(frozen=True)
class Value__Boolean(Value):
    value: st.bool


@dataclass(frozen=True)
class Value__ExternalInstance(Value):
    value: "ExternalInstance"


@dataclass(frozen=True)
class Value__Dictionary(Value):
    value: "Dictionary"


@dataclass(frozen=True)
class Value__Pattern(Value):
    value: "Pattern"


@dataclass(frozen=True)
class Value__Call(Value):
    value: "Call"


@dataclass(frozen=True)
class Value__List(Value):
    value: typing.Sequence["Value"]


@dataclass(frozen=True)
class Value__Variable(Value):
    value: str


@dataclass(frozen=True)
class Value__RestVariable(Value):
    value: str


@dataclass(frozen=True)
class Value__Expression(Value):
    value: "Operation"


@dataclass(frozen=True)
class Value__Partial(Value):
    value: "Partial"

Value.VARIANTS_MAP = {
    "Number": Value__Number,
    "String": Value__String,
    "Boolean": Value__Boolean,
    "ExternalInstance": Value__ExternalInstance,
    "Dictionary": Value__Dictionary,
    "Pattern": Value__Pattern,
    "Call": Value__Call,
    "List": Value__List,
    "Variable": Value__Variable,
    "RestVariable": Value__RestVariable,
    "Expression": Value__Expression,
    "Partial": Value__Partial,
}

