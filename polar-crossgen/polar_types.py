# pyre-strict
from dataclasses import dataclass
import typing
import serde_types as st

@dataclass(frozen=True)
class Call:
    name: "Symbol"
    args: typing.Sequence["Term"]
    kwargs: typing.Optional[typing.Dict["Symbol", "Term"]]


@dataclass(frozen=True)
class Dictionary:
    fields: typing.Dict["Symbol", "Term"]


@dataclass(frozen=True)
class ExternalInstance:
    instance_id: st.uint64
    constructor: typing.Optional["Term"]
    repr: typing.Optional[str]


@dataclass(frozen=True)
class InstanceLiteral:
    tag: "Symbol"
    fields: "Dictionary"


class Node:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Node]]


@dataclass(frozen=True)
class Node__Rule(Node):
    NAME = "Rule"  # type: str
    INDEX = 0  # type: int
    value: "Rule"


@dataclass(frozen=True)
class Node__Term(Node):
    NAME = "Term"  # type: str
    INDEX = 1  # type: int
    value: "Term"

Node.VARIANTS = [
    Node__Rule,
    Node__Term,
]

Node.VARIANTS_MAP = {
    "Rule": Node__Rule,
    "Term": Node__Term,
}


class Numeric:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Numeric]]


@dataclass(frozen=True)
class Numeric__Integer(Numeric):
    NAME = "Integer"  # type: str
    INDEX = 0  # type: int
    value: st.int64


@dataclass(frozen=True)
class Numeric__Float(Numeric):
    NAME = "Float"  # type: str
    INDEX = 1  # type: int
    value: st.float64

Numeric.VARIANTS = [
    Numeric__Integer,
    Numeric__Float,
]

Numeric.VARIANTS_MAP = {
    "Integer": Numeric__Integer,
    "Float": Numeric__Float,
}


@dataclass(frozen=True)
class Operation:
    operator: "Operator"
    args: typing.Sequence["Term"]


class Operator:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Operator]]


@dataclass(frozen=True)
class Operator__Debug(Operator):
    NAME = "Debug"  # type: str
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class Operator__Print(Operator):
    NAME = "Print"  # type: str
    INDEX = 1  # type: int
    pass


@dataclass(frozen=True)
class Operator__Cut(Operator):
    NAME = "Cut"  # type: str
    INDEX = 2  # type: int
    pass


@dataclass(frozen=True)
class Operator__In(Operator):
    NAME = "In"  # type: str
    INDEX = 3  # type: int
    pass


@dataclass(frozen=True)
class Operator__Isa(Operator):
    NAME = "Isa"  # type: str
    INDEX = 4  # type: int
    pass


@dataclass(frozen=True)
class Operator__New(Operator):
    NAME = "New"  # type: str
    INDEX = 5  # type: int
    pass


@dataclass(frozen=True)
class Operator__Dot(Operator):
    NAME = "Dot"  # type: str
    INDEX = 6  # type: int
    pass


@dataclass(frozen=True)
class Operator__Not(Operator):
    NAME = "Not"  # type: str
    INDEX = 7  # type: int
    pass


@dataclass(frozen=True)
class Operator__Mul(Operator):
    NAME = "Mul"  # type: str
    INDEX = 8  # type: int
    pass


@dataclass(frozen=True)
class Operator__Div(Operator):
    NAME = "Div"  # type: str
    INDEX = 9  # type: int
    pass


@dataclass(frozen=True)
class Operator__Mod(Operator):
    NAME = "Mod"  # type: str
    INDEX = 10  # type: int
    pass


@dataclass(frozen=True)
class Operator__Rem(Operator):
    NAME = "Rem"  # type: str
    INDEX = 11  # type: int
    pass


@dataclass(frozen=True)
class Operator__Add(Operator):
    NAME = "Add"  # type: str
    INDEX = 12  # type: int
    pass


@dataclass(frozen=True)
class Operator__Sub(Operator):
    NAME = "Sub"  # type: str
    INDEX = 13  # type: int
    pass


@dataclass(frozen=True)
class Operator__Eq(Operator):
    NAME = "Eq"  # type: str
    INDEX = 14  # type: int
    pass


@dataclass(frozen=True)
class Operator__Geq(Operator):
    NAME = "Geq"  # type: str
    INDEX = 15  # type: int
    pass


@dataclass(frozen=True)
class Operator__Leq(Operator):
    NAME = "Leq"  # type: str
    INDEX = 16  # type: int
    pass


@dataclass(frozen=True)
class Operator__Neq(Operator):
    NAME = "Neq"  # type: str
    INDEX = 17  # type: int
    pass


@dataclass(frozen=True)
class Operator__Gt(Operator):
    NAME = "Gt"  # type: str
    INDEX = 18  # type: int
    pass


@dataclass(frozen=True)
class Operator__Lt(Operator):
    NAME = "Lt"  # type: str
    INDEX = 19  # type: int
    pass


@dataclass(frozen=True)
class Operator__Unify(Operator):
    NAME = "Unify"  # type: str
    INDEX = 20  # type: int
    pass


@dataclass(frozen=True)
class Operator__Or(Operator):
    NAME = "Or"  # type: str
    INDEX = 21  # type: int
    pass


@dataclass(frozen=True)
class Operator__And(Operator):
    NAME = "And"  # type: str
    INDEX = 22  # type: int
    pass


@dataclass(frozen=True)
class Operator__ForAll(Operator):
    NAME = "ForAll"  # type: str
    INDEX = 23  # type: int
    pass


@dataclass(frozen=True)
class Operator__Assign(Operator):
    NAME = "Assign"  # type: str
    INDEX = 24  # type: int
    pass

Operator.VARIANTS = [
    Operator__Debug,
    Operator__Print,
    Operator__Cut,
    Operator__In,
    Operator__Isa,
    Operator__New,
    Operator__Dot,
    Operator__Not,
    Operator__Mul,
    Operator__Div,
    Operator__Mod,
    Operator__Rem,
    Operator__Add,
    Operator__Sub,
    Operator__Eq,
    Operator__Geq,
    Operator__Leq,
    Operator__Neq,
    Operator__Gt,
    Operator__Lt,
    Operator__Unify,
    Operator__Or,
    Operator__And,
    Operator__ForAll,
    Operator__Assign,
]

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
    parameter: "Term"
    specializer: typing.Optional["Term"]


@dataclass(frozen=True)
class Partial:
    constraints: typing.Sequence["Operation"]
    variable: "Symbol"


class Pattern:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Pattern]]


@dataclass(frozen=True)
class Pattern__Dictionary(Pattern):
    NAME = "Dictionary"  # type: str
    INDEX = 0  # type: int
    value: "Dictionary"


@dataclass(frozen=True)
class Pattern__Instance(Pattern):
    NAME = "Instance"  # type: str
    INDEX = 1  # type: int
    value: "InstanceLiteral"

Pattern.VARIANTS = [
    Pattern__Dictionary,
    Pattern__Instance,
]

Pattern.VARIANTS_MAP = {
    "Dictionary": Pattern__Dictionary,
    "Instance": Pattern__Instance,
}


class QueryEvent:
    VARIANTS = []  # type: typing.Sequence[typing.Type[QueryEvent]]


@dataclass(frozen=True)
class QueryEvent__None(QueryEvent):
    NAME = "None"  # type: str
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class QueryEvent__Done(QueryEvent):
    NAME = "Done"  # type: str
    INDEX = 1  # type: int
    result: st.bool


@dataclass(frozen=True)
class QueryEvent__Debug(QueryEvent):
    NAME = "Debug"  # type: str
    INDEX = 2  # type: int
    message: str


@dataclass(frozen=True)
class QueryEvent__MakeExternal(QueryEvent):
    NAME = "MakeExternal"  # type: str
    INDEX = 3  # type: int
    instance_id: st.uint64
    constructor: "Term"


@dataclass(frozen=True)
class QueryEvent__ExternalCall(QueryEvent):
    NAME = "ExternalCall"  # type: str
    INDEX = 4  # type: int
    call_id: st.uint64
    instance: "Term"
    attribute: "Symbol"
    args: typing.Optional[typing.Sequence["Term"]]
    kwargs: typing.Optional[typing.Dict["Symbol", "Term"]]


@dataclass(frozen=True)
class QueryEvent__ExternalIsa(QueryEvent):
    NAME = "ExternalIsa"  # type: str
    INDEX = 5  # type: int
    call_id: st.uint64
    instance: "Term"
    class_tag: "Symbol"


@dataclass(frozen=True)
class QueryEvent__ExternalIsSubSpecializer(QueryEvent):
    NAME = "ExternalIsSubSpecializer"  # type: str
    INDEX = 6  # type: int
    call_id: st.uint64
    instance_id: st.uint64
    left_class_tag: "Symbol"
    right_class_tag: "Symbol"


@dataclass(frozen=True)
class QueryEvent__ExternalIsSubclass(QueryEvent):
    NAME = "ExternalIsSubclass"  # type: str
    INDEX = 7  # type: int
    call_id: st.uint64
    left_class_tag: "Symbol"
    right_class_tag: "Symbol"


@dataclass(frozen=True)
class QueryEvent__ExternalUnify(QueryEvent):
    NAME = "ExternalUnify"  # type: str
    INDEX = 8  # type: int
    call_id: st.uint64
    left_instance_id: st.uint64
    right_instance_id: st.uint64


@dataclass(frozen=True)
class QueryEvent__Result(QueryEvent):
    NAME = "Result"  # type: str
    INDEX = 9  # type: int
    bindings: typing.Dict["Symbol", "Term"]
    trace: typing.Optional["TraceResult"]


@dataclass(frozen=True)
class QueryEvent__ExternalOp(QueryEvent):
    NAME = "ExternalOp"  # type: str
    INDEX = 10  # type: int
    call_id: st.uint64
    operator: "Operator"
    args: typing.Sequence["Term"]


@dataclass(frozen=True)
class QueryEvent__NextExternal(QueryEvent):
    NAME = "NextExternal"  # type: str
    INDEX = 11  # type: int
    call_id: st.uint64
    iterable: "Term"

QueryEvent.VARIANTS = [
    QueryEvent__None,
    QueryEvent__Done,
    QueryEvent__Debug,
    QueryEvent__MakeExternal,
    QueryEvent__ExternalCall,
    QueryEvent__ExternalIsa,
    QueryEvent__ExternalIsSubSpecializer,
    QueryEvent__ExternalIsSubclass,
    QueryEvent__ExternalUnify,
    QueryEvent__Result,
    QueryEvent__ExternalOp,
    QueryEvent__NextExternal,
]

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
    name: "Symbol"
    params: typing.Sequence["Parameter"]
    body: "Term"


@dataclass(frozen=True)
class Symbol:
    value: str


@dataclass(frozen=True)
class Term:
    value: "Value"


@dataclass(frozen=True)
class Trace:
    node: "Node"
    children: typing.Sequence["Trace"]


@dataclass(frozen=True)
class TraceResult:
    trace: "Trace"
    formatted: str


class Value:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Value]]


@dataclass(frozen=True)
class Value__Number(Value):
    NAME = "Number"  # type: str
    INDEX = 0  # type: int
    value: "Numeric"


@dataclass(frozen=True)
class Value__String(Value):
    NAME = "String"  # type: str
    INDEX = 1  # type: int
    value: str


@dataclass(frozen=True)
class Value__Boolean(Value):
    NAME = "Boolean"  # type: str
    INDEX = 2  # type: int
    value: st.bool


@dataclass(frozen=True)
class Value__ExternalInstance(Value):
    NAME = "ExternalInstance"  # type: str
    INDEX = 3  # type: int
    value: "ExternalInstance"


@dataclass(frozen=True)
class Value__InstanceLiteral(Value):
    NAME = "InstanceLiteral"  # type: str
    INDEX = 4  # type: int
    value: "InstanceLiteral"


@dataclass(frozen=True)
class Value__Dictionary(Value):
    NAME = "Dictionary"  # type: str
    INDEX = 5  # type: int
    value: "Dictionary"


@dataclass(frozen=True)
class Value__Pattern(Value):
    NAME = "Pattern"  # type: str
    INDEX = 6  # type: int
    value: "Pattern"


@dataclass(frozen=True)
class Value__Call(Value):
    NAME = "Call"  # type: str
    INDEX = 7  # type: int
    value: "Call"


@dataclass(frozen=True)
class Value__List(Value):
    NAME = "List"  # type: str
    INDEX = 8  # type: int
    value: typing.Sequence["Term"]


@dataclass(frozen=True)
class Value__Variable(Value):
    NAME = "Variable"  # type: str
    INDEX = 9  # type: int
    value: "Symbol"


@dataclass(frozen=True)
class Value__RestVariable(Value):
    NAME = "RestVariable"  # type: str
    INDEX = 10  # type: int
    value: "Symbol"


@dataclass(frozen=True)
class Value__Expression(Value):
    NAME = "Expression"  # type: str
    INDEX = 11  # type: int
    value: "Operation"


@dataclass(frozen=True)
class Value__Partial(Value):
    NAME = "Partial"  # type: str
    INDEX = 12  # type: int
    value: "Partial"

Value.VARIANTS = [
    Value__Number,
    Value__String,
    Value__Boolean,
    Value__ExternalInstance,
    Value__InstanceLiteral,
    Value__Dictionary,
    Value__Pattern,
    Value__Call,
    Value__List,
    Value__Variable,
    Value__RestVariable,
    Value__Expression,
    Value__Partial,
]

Value.VARIANTS_MAP = {
    "Number": Value__Number,
    "String": Value__String,
    "Boolean": Value__Boolean,
    "ExternalInstance": Value__ExternalInstance,
    "InstanceLiteral": Value__InstanceLiteral,
    "Dictionary": Value__Dictionary,
    "Pattern": Value__Pattern,
    "Call": Value__Call,
    "List": Value__List,
    "Variable": Value__Variable,
    "RestVariable": Value__RestVariable,
    "Expression": Value__Expression,
    "Partial": Value__Partial,
}

