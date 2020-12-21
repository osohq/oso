# {
#     "Call": Struct(
#         [
#             Named {
#                 name: "name",
#                 value: TypeName(
#                     "Symbol",
#                 ),
#             },
#             Named {
#                 name: "args",
#                 value: Seq(
#                     TypeName(
#                         "Term",
#                     ),
#                 ),
#             },
#             Named {
#                 name: "kwargs",
#                 value: Option(
#                     Map {
#                         key: TypeName(
#                             "Symbol",
#                         ),
#                         value: TypeName(
#                             "Term",
#                         ),
#                     },
#                 ),
#             },
#         ],
#     ),
#     "Dictionary": Struct(
#         [
#             Named {
#                 name: "fields",
#                 value: Map {
#                     key: TypeName(
#                         "Symbol",
#                     ),
#                     value: TypeName(
#                         "Term",
#                     ),
#                 },
#             },
#         ],
#     ),
#     "ExternalInstance": Struct(
#         [
#             Named {
#                 name: "instance_id",
#                 value: U64,
#             },
#             Named {
#                 name: "constructor",
#                 value: Option(
#                     TypeName(
#                         "Term",
#                     ),
#                 ),
#             },
#             Named {
#                 name: "repr",
#                 value: Option(
#                     Str,
#                 ),
#             },
#         ],
#     ),
#     "InstanceLiteral": Struct(
#         [
#             Named {
#                 name: "tag",
#                 value: TypeName(
#                     "Symbol",
#                 ),
#             },
#             Named {
#                 name: "fields",
#                 value: TypeName(
#                     "Dictionary",
#                 ),
#             },
#         ],
#     ),
#     "Numeric": Enum(
#         {
#             0: Named {
#                 name: "Integer",
#                 value: NewType(
#                     I64,
#                 ),
#             },
#             1: Named {
#                 name: "Float",
#                 value: NewType(
#                     F64,
#                 ),
#             },
#         },
#     ),
#     "Operation": Struct(
#         [
#             Named {
#                 name: "operator",
#                 value: TypeName(
#                     "Operator",
#                 ),
#             },
#             Named {
#                 name: "args",
#                 value: Seq(
#                     TypeName(
#                         "Term",
#                     ),
#                 ),
#             },
#         ],
#     ),
#     "Operator": Enum(
#         {
#             0: Named {
#                 name: "Debug",
#                 value: Unit,
#             },
#             1: Named {
#                 name: "Print",
#                 value: Unit,
#             },
#             2: Named {
#                 name: "Cut",
#                 value: Unit,
#             },
#             3: Named {
#                 name: "In",
#                 value: Unit,
#             },
#             4: Named {
#                 name: "Isa",
#                 value: Unit,
#             },
#             5: Named {
#                 name: "New",
#                 value: Unit,
#             },
#             6: Named {
#                 name: "Dot",
#                 value: Unit,
#             },
#             7: Named {
#                 name: "Not",
#                 value: Unit,
#             },
#             8: Named {
#                 name: "Mul",
#                 value: Unit,
#             },
#             9: Named {
#                 name: "Div",
#                 value: Unit,
#             },
#             10: Named {
#                 name: "Mod",
#                 value: Unit,
#             },
#             11: Named {
#                 name: "Rem",
#                 value: Unit,
#             },
#             12: Named {
#                 name: "Add",
#                 value: Unit,
#             },
#             13: Named {
#                 name: "Sub",
#                 value: Unit,
#             },
#             14: Named {
#                 name: "Eq",
#                 value: Unit,
#             },
#             15: Named {
#                 name: "Geq",
#                 value: Unit,
#             },
#             16: Named {
#                 name: "Leq",
#                 value: Unit,
#             },
#             17: Named {
#                 name: "Neq",
#                 value: Unit,
#             },
#             18: Named {
#                 name: "Gt",
#                 value: Unit,
#             },
#             19: Named {
#                 name: "Lt",
#                 value: Unit,
#             },
#             20: Named {
#                 name: "Unify",
#                 value: Unit,
#             },
#             21: Named {
#                 name: "Or",
#                 value: Unit,
#             },
#             22: Named {
#                 name: "And",
#                 value: Unit,
#             },
#             23: Named {
#                 name: "ForAll",
#                 value: Unit,
#             },
#             24: Named {
#                 name: "Assign",
#                 value: Unit,
#             },
#         },
#     ),
#     "Partial": Struct(
#         [
#             Named {
#                 name: "constraints",
#                 value: Seq(
#                     TypeName(
#                         "Operation",
#                     ),
#                 ),
#             },
#             Named {
#                 name: "variable",
#                 value: TypeName(
#                     "Symbol",
#                 ),
#             },
#         ],
#     ),
#     "Pattern": Enum(
#         {
#             0: Named {
#                 name: "Dictionary",
#                 value: NewType(
#                     TypeName(
#                         "Dictionary",
#                     ),
#                 ),
#             },
#             1: Named {
#                 name: "Instance",
#                 value: NewType(
#                     TypeName(
#                         "InstanceLiteral",
#                     ),
#                 ),
#             },
#         },
#     ),
#     "Symbol": NewTypeStruct(
#         Str,
#     ),
#     "Term": Struct(
#         [
#             Named {
#                 name: "value",
#                 value: TypeName(
#                     "Value",
#                 ),
#             },
#         ],
#     ),
#     "Value": Enum(
#         {
#             0: Named {
#                 name: "Number",
#                 value: NewType(
#                     TypeName(
#                         "Numeric",
#                     ),
#                 ),
#             },
#             1: Named {
#                 name: "String",
#                 value: NewType(
#                     Str,
#                 ),
#             },
#             2: Named {
#                 name: "Boolean",
#                 value: NewType(
#                     Bool,
#                 ),
#             },
#             3: Named {
#                 name: "ExternalInstance",
#                 value: NewType(
#                     TypeName(
#                         "ExternalInstance",
#                     ),
#                 ),
#             },
#             4: Named {
#                 name: "InstanceLiteral",
#                 value: NewType(
#                     TypeName(
#                         "InstanceLiteral",
#                     ),
#                 ),
#             },
#             5: Named {
#                 name: "Dictionary",
#                 value: NewType(
#                     TypeName(
#                         "Dictionary",
#                     ),
#                 ),
#             },
#             6: Named {
#                 name: "Pattern",
#                 value: NewType(
#                     TypeName(
#                         "Pattern",
#                     ),
#                 ),
#             },
#             7: Named {
#                 name: "Call",
#                 value: NewType(
#                     TypeName(
#                         "Call",
#                     ),
#                 ),
#             },
#             8: Named {
#                 name: "List",
#                 value: NewType(
#                     Seq(
#                         TypeName(
#                             "Term",
#                         ),
#                     ),
#                 ),
#             },
#             9: Named {
#                 name: "Variable",
#                 value: NewType(
#                     TypeName(
#                         "Symbol",
#                     ),
#                 ),
#             },
#             10: Named {
#                 name: "RestVariable",
#                 value: NewType(
#                     TypeName(
#                         "Symbol",
#                     ),
#                 ),
#             },
#             11: Named {
#                 name: "Expression",
#                 value: NewType(
#                     TypeName(
#                         "Operation",
#                     ),
#                 ),
#             },
#             12: Named {
#                 name: "Partial",
#                 value: NewType(
#                     TypeName(
#                         "Partial",
#                     ),
#                 ),
#             },
#         },
#     ),
# }
# pyre-strict
from dataclasses import dataclass
import typing
import serde_types as st
import serde_json as sj

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


@dataclass(frozen=True)
class Symbol:
    value: str


@dataclass(frozen=True)
class Term:
    value: "Value"


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

assert sj.deserialize_json('{"String": "abc"}', Value) == Value__String("abc")
assert sj.deserialize_json('{"Number": {"Integer": 12}}', Value) == Value__Number(Numeric__Integer(12))
# assert sj.deserialize_json('{"String": "abc"}', Value) == Value__String("abc")
# assert sj.deserialize_json('{"String": "abc"}', Value) == Value__String("abc")