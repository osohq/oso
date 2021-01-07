import dataclasses
import collections
import typing
from typing import get_type_hints
import json


class DeserializationError(Exception):
    pass


class SerializationError(Exception):
    pass


class Enum:
    def __getattribute__(self, attr):
        content = object.__getattribute__(self, "content")
        return content.__getattribute__(attr)

    def __str__(self):
        name = object.__getattribute__(self, "_name")
        return f"{name}({self.__str__()})"

    def __repr__(self):
        name = object.__getattribute__(self, "_name")
        return f"{name}({self.__repr__()})"

    def __eq__(self, other):
        return self.__eq__(other)


def make_special_method_wrapper(method_name):
    def wrapper(self, *args, **kwargs):
        return getattr(self, method_name)(*args, **kwargs)

    wrapper.__name__ = method_name
    return wrapper


def get_newtype_hints(obj_type):
    # hack to get the forward-evaluated version of a typing type
    def __types() -> obj_type:
        pass

    return get_type_hints(__types)["return"]


class TypedJsonDeserializer:
    """
    Deserializes JSON using the provided type


    JSON | Python
    ---------------
    object | dict
    array | list
    string | str
    number (int) | int
    number (real) | float
    true | True
    false | False
    null | None
    """

    input: typing.Any
    primitive_types = [str, int, float, bool]
    name: str = ""

    def __init__(self, input_val: typing.Any, name: str = ""):
        self.input = input_val
        self.name = name

    def deserialize_any(self, obj_type) -> typing.Any:
        if obj_type in self.primitive_types:
            return obj_type(self.input)
        elif hasattr(obj_type, "__origin__"):  # Generic type
            types = getattr(obj_type, "__args__")
            if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
                assert len(types) == 1
                item_type = types[0]
                assert isinstance(self.input, list)
                result = []
                for item in self.input:
                    result.append(
                        TypedJsonDeserializer(item).deserialize_any(item_type)
                    )

                return result

            elif getattr(obj_type, "__origin__") == tuple:  # Tuple
                result = []
                assert isinstance(self.input, list)
                assert len(types) == len(self.input)
                for (item_type, item) in zip(types, self.input):
                    item = TypedJsonDeserializer(item).deserialize_any(item_type)
                    result.append(item)
                return obj_type(result)

            elif (
                getattr(obj_type, "__origin__") == typing.Union
            ):  # Option or enum variant
                if (
                    len(types) == 2
                    and isinstance(types[1], type)
                    and isinstance(None, types[1])
                ):  # Option
                    if self.input is None:
                        return None
                    else:
                        return self.deserialize_any(types[0])
            elif getattr(obj_type, "__origin__") == dict:  # Map
                assert len(types) == 2
                result_dict = dict()
                input_dict = self.input or {}
                assert isinstance(input_dict, dict)
                for k, v in input_dict.items():
                    result_dict[k] = TypedJsonDeserializer(v).deserialize_any(types[1])

                return result_dict

            else:
                raise DeserializationError("Unexpected type", obj_type)

        else:
            # handle enums + structs

            types = get_type_hints(obj_type)
            if dataclasses.is_dataclass(obj_type):
                fields = dataclasses.fields(obj_type)
                # regular struct
                kwargs = {
                    field.name: TypedJsonDeserializer(
                        self.input.get(field.name, None)
                    ).deserialize_any(types[field.name])
                    for field in fields
                }
                return obj_type(**kwargs)
            elif not isinstance(obj_type, type):
                # newtype
                ty = get_newtype_hints(obj_type.__supertype__)
                return obj_type(self.deserialize_any(ty))
            elif issubclass(obj_type, Enum):
                # enum
                assert (
                    len(self.input) == 1
                ), "deserializing an enum variant should only have one k: v pair"
                variant_name, variant_value = next(iter(self.input.items()))
                type_name = obj_type._name + variant_name
                for t in types["content"].__args__:
                    if t.__name__ == type_name:
                        return obj_type(
                            tag=variant_name,
                            content=TypedJsonDeserializer(
                                variant_value
                            ).deserialize_any(t),
                        )
                raise DeserializationError(
                    f"unexpected variant for {obj_type}", variant_name
                )
            else:
                # type is a struct, but not a dataclass
                # no idea how to handle
                raise DeserializationError("Unexpected type", obj_type)


class TypedJsonSerializer:
    """
    Serializes JSON using the provided type


    JSON | Python
    ---------------
    object | dict
    array | list
    string | str
    number (int) | int
    number (real) | float
    true | True
    false | False
    null | None
    """

    input: typing.Any
    primitive_types = [str, int, float, bool]
    name: str = ""

    def __init__(self, input_val: typing.Any, name: str = ""):
        self.input = input_val
        self.name = name

    def serialize_any(self, obj_type) -> typing.Any:
        if obj_type in self.primitive_types:
            return obj_type(self.input)
        elif hasattr(obj_type, "__origin__"):  # Generic type
            types = getattr(obj_type, "__args__")
            if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
                assert len(types) == 1
                item_type = types[0]
                assert isinstance(self.input, list)
                result = []
                for item in self.input:
                    result.append(TypedJsonSerializer(item).serialize_any(item_type))

                return result

            elif getattr(obj_type, "__origin__") == tuple:  # Tuple
                result = []
                assert len(types) == len(self.input)
                for (item_type, item) in zip(types, self.input):
                    item = TypedJsonSerializer(item).serialize_any(item_type)
                    result.append(item)
                return obj_type(result)

            elif (
                getattr(obj_type, "__origin__") == typing.Union
            ):  # Option or enum variant
                if (
                    len(types) == 2
                    and isinstance(types[1], type)
                    and isinstance(None, types[1])
                ):  # Option
                    if self.input is None:
                        return None
                    else:
                        return self.serialize_any(types[0])
            elif getattr(obj_type, "__origin__") == dict:  # Map
                assert len(types) == 2
                result_dict = dict()
                input_dict = self.input or {}
                assert isinstance(input_dict, dict)
                for k, v in input_dict.items():
                    result_dict[k] = TypedJsonSerializer(v).serialize_any(types[1])

                return result_dict

            else:
                raise SerializationError("Unexpected type", obj_type)

        else:
            # handle enums + structs

            types = get_type_hints(obj_type)
            if dataclasses.is_dataclass(obj_type):
                fields = dataclasses.fields(obj_type)
                # regular struct
                kwargs = {
                    field.name: TypedJsonSerializer(
                        getattr(self.input, field.name, None)
                    ).serialize_any(types[field.name])
                    for field in fields
                }
                return kwargs
            elif not isinstance(obj_type, type):
                # newtype
                ty = get_newtype_hints(obj_type.__supertype__)
                return self.serialize_any(ty)
            elif issubclass(obj_type, Enum):
                # enum
                content = object.__getattribute__(self.input, "content")
                tag = object.__getattribute__(self.input, "tag")
                type_name = obj_type._name + tag
                for t in types["content"].__args__:
                    if t.__name__ == type_name:
                        return {tag: TypedJsonSerializer(content).serialize_any(t)}
                raise SerializationError(f"unexpected variant for {obj_type}", content)
            else:
                # type is a struct, but not a dataclass
                # no idea how to handle
                raise SerializationError("Unexpected type", obj_type)


def deserialize_json(input_str, obj):
    deserializer = TypedJsonDeserializer(json.loads(input_str))
    return deserializer.deserialize_any(obj)


def serialize_json(input_val, obj):
    serializer = TypedJsonSerializer(input_val)
    output = serializer.serialize_any(obj)
    return json.dumps(output)


@dataclasses.dataclass(frozen=True)
class Call:
    name: str
    args: typing.Sequence["Value"]
    kwargs: typing.Optional[typing.Dict[str, "Value"]]


@dataclasses.dataclass(frozen=True)
class Dictionary:
    fields: typing.Dict[str, "Value"]


ErrorKindParse = typing.NewType("ErrorKindParse", "ParseError")


ErrorKindRuntime = typing.NewType("ErrorKindRuntime", "RuntimeError")


ErrorKindOperational = typing.NewType("ErrorKindOperational", "OperationalError")


ErrorKindParameter = typing.NewType("ErrorKindParameter", "ParameterError")


class ErrorKind(Enum):
    content: typing.Union[
        "ErrorKindParse",
        "ErrorKindRuntime",
        "ErrorKindOperational",
        "ErrorKindParameter",
    ]
    tag: typing.Literal[
        "Parse",
        "Runtime",
        "Operational",
        "Parameter",
    ]
    _name: str = "ErrorKind"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


@dataclasses.dataclass(frozen=True)
class ExternalInstance:
    instance_id: int
    constructor: typing.Optional["Value"]
    repr: typing.Optional[str]


@dataclasses.dataclass(frozen=True)
class FormattedPolarError:
    kind: "ErrorKind"
    formatted: str


@dataclasses.dataclass(frozen=True)
class InstanceLiteral:
    tag: str
    fields: "Dictionary"


@dataclasses.dataclass(frozen=True)
class Message:
    kind: "MessageKind"
    msg: str


class MessageKindPrint:
    pass


class MessageKindWarning:
    pass


class MessageKind(Enum):
    content: typing.Union[
        "MessageKindPrint",
        "MessageKindWarning",
    ]
    tag: typing.Literal[
        "Print",
        "Warning",
    ]
    _name: str = "MessageKind"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


NodeRule = typing.NewType("NodeRule", "Rule")


NodeTerm = typing.NewType("NodeTerm", "Value")


class Node(Enum):
    content: typing.Union[
        "NodeRule",
        "NodeTerm",
    ]
    tag: typing.Literal[
        "Rule",
        "Term",
    ]
    _name: str = "Node"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


NumericInteger = typing.NewType("NumericInteger", int)


NumericFloat = typing.NewType("NumericFloat", float)


class Numeric(Enum):
    content: typing.Union[
        "NumericInteger",
        "NumericFloat",
    ]
    tag: typing.Literal[
        "Integer",
        "Float",
    ]
    _name: str = "Numeric"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


@dataclasses.dataclass(frozen=True)
class Operation:
    operator: "Operator"
    args: typing.Sequence["Value"]


OperationalErrorUnimplemented = typing.NewType("OperationalErrorUnimplemented", str)


class OperationalErrorUnknown:
    pass


OperationalErrorInvalidState = typing.NewType("OperationalErrorInvalidState", str)


class OperationalError(Enum):
    content: typing.Union[
        "OperationalErrorUnimplemented",
        "OperationalErrorUnknown",
        "OperationalErrorInvalidState",
    ]
    tag: typing.Literal[
        "Unimplemented",
        "Unknown",
        "InvalidState",
    ]
    _name: str = "OperationalError"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


class OperatorDebug:
    pass


class OperatorPrint:
    pass


class OperatorCut:
    pass


class OperatorIn:
    pass


class OperatorIsa:
    pass


class OperatorNew:
    pass


class OperatorDot:
    pass


class OperatorNot:
    pass


class OperatorMul:
    pass


class OperatorDiv:
    pass


class OperatorMod:
    pass


class OperatorRem:
    pass


class OperatorAdd:
    pass


class OperatorSub:
    pass


class OperatorEq:
    pass


class OperatorGeq:
    pass


class OperatorLeq:
    pass


class OperatorNeq:
    pass


class OperatorGt:
    pass


class OperatorLt:
    pass


class OperatorUnify:
    pass


class OperatorOr:
    pass


class OperatorAnd:
    pass


class OperatorForAll:
    pass


class OperatorAssign:
    pass


class Operator(Enum):
    content: typing.Union[
        "OperatorDebug",
        "OperatorPrint",
        "OperatorCut",
        "OperatorIn",
        "OperatorIsa",
        "OperatorNew",
        "OperatorDot",
        "OperatorNot",
        "OperatorMul",
        "OperatorDiv",
        "OperatorMod",
        "OperatorRem",
        "OperatorAdd",
        "OperatorSub",
        "OperatorEq",
        "OperatorGeq",
        "OperatorLeq",
        "OperatorNeq",
        "OperatorGt",
        "OperatorLt",
        "OperatorUnify",
        "OperatorOr",
        "OperatorAnd",
        "OperatorForAll",
        "OperatorAssign",
    ]
    tag: typing.Literal[
        "Debug",
        "Print",
        "Cut",
        "In",
        "Isa",
        "New",
        "Dot",
        "Not",
        "Mul",
        "Div",
        "Mod",
        "Rem",
        "Add",
        "Sub",
        "Eq",
        "Geq",
        "Leq",
        "Neq",
        "Gt",
        "Lt",
        "Unify",
        "Or",
        "And",
        "ForAll",
        "Assign",
    ]
    _name: str = "Operator"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


@dataclasses.dataclass(frozen=True)
class Parameter:
    parameter: "Value"
    specializer: typing.Optional["Value"]


ParameterError = typing.NewType("ParameterError", str)


@dataclasses.dataclass(frozen=True)
class ParseErrorIntegerOverflow:
    token: str
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorInvalidTokenCharacter:
    token: str
    c: str
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorInvalidToken:
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorUnrecognizedEOF:
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorUnrecognizedToken:
    token: str
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorExtraToken:
    token: str
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorReservedWord:
    token: str
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorInvalidFloat:
    token: str
    loc: int


@dataclasses.dataclass(frozen=True)
class ParseErrorWrongValueType:
    loc: int
    term: "Value"
    expected: str


class ParseError(Enum):
    content: typing.Union[
        "ParseErrorIntegerOverflow",
        "ParseErrorInvalidTokenCharacter",
        "ParseErrorInvalidToken",
        "ParseErrorUnrecognizedEOF",
        "ParseErrorUnrecognizedToken",
        "ParseErrorExtraToken",
        "ParseErrorReservedWord",
        "ParseErrorInvalidFloat",
        "ParseErrorWrongValueType",
    ]
    tag: typing.Literal[
        "IntegerOverflow",
        "InvalidTokenCharacter",
        "InvalidToken",
        "UnrecognizedEOF",
        "UnrecognizedToken",
        "ExtraToken",
        "ReservedWord",
        "InvalidFloat",
        "WrongValueType",
    ]
    _name: str = "ParseError"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


@dataclasses.dataclass(frozen=True)
class Partial:
    constraints: typing.Sequence["Operation"]
    variable: str


PatternDictionary = typing.NewType("PatternDictionary", "Dictionary")


PatternInstance = typing.NewType("PatternInstance", "InstanceLiteral")


class Pattern(Enum):
    content: typing.Union[
        "PatternDictionary",
        "PatternInstance",
    ]
    tag: typing.Literal[
        "Dictionary",
        "Instance",
    ]
    _name: str = "Pattern"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


class QueryEventNone:
    pass


@dataclasses.dataclass(frozen=True)
class QueryEventDone:
    result: bool


@dataclasses.dataclass(frozen=True)
class QueryEventDebug:
    message: str


@dataclasses.dataclass(frozen=True)
class QueryEventMakeExternal:
    instance_id: int
    constructor: "Value"


@dataclasses.dataclass(frozen=True)
class QueryEventExternalCall:
    call_id: int
    instance: "Value"
    attribute: str
    args: typing.Optional[typing.Sequence["Value"]]
    kwargs: typing.Optional[typing.Dict[str, "Value"]]


@dataclasses.dataclass(frozen=True)
class QueryEventExternalIsa:
    call_id: int
    instance: "Value"
    class_tag: str


@dataclasses.dataclass(frozen=True)
class QueryEventExternalIsSubSpecializer:
    call_id: int
    instance_id: int
    left_class_tag: str
    right_class_tag: str


@dataclasses.dataclass(frozen=True)
class QueryEventExternalIsSubclass:
    call_id: int
    left_class_tag: str
    right_class_tag: str


@dataclasses.dataclass(frozen=True)
class QueryEventExternalUnify:
    call_id: int
    left_instance_id: int
    right_instance_id: int


@dataclasses.dataclass(frozen=True)
class QueryEventResult:
    bindings: typing.Dict[str, "Value"]
    trace: typing.Optional["TraceResult"]


@dataclasses.dataclass(frozen=True)
class QueryEventExternalOp:
    call_id: int
    operator: "Operator"
    args: typing.Sequence["Value"]


@dataclasses.dataclass(frozen=True)
class QueryEventNextExternal:
    call_id: int
    iterable: "Value"


class QueryEvent(Enum):
    content: typing.Union[
        "QueryEventNone",
        "QueryEventDone",
        "QueryEventDebug",
        "QueryEventMakeExternal",
        "QueryEventExternalCall",
        "QueryEventExternalIsa",
        "QueryEventExternalIsSubSpecializer",
        "QueryEventExternalIsSubclass",
        "QueryEventExternalUnify",
        "QueryEventResult",
        "QueryEventExternalOp",
        "QueryEventNextExternal",
    ]
    tag: typing.Literal[
        "None",
        "Done",
        "Debug",
        "MakeExternal",
        "ExternalCall",
        "ExternalIsa",
        "ExternalIsSubSpecializer",
        "ExternalIsSubclass",
        "ExternalUnify",
        "Result",
        "ExternalOp",
        "NextExternal",
    ]
    _name: str = "QueryEvent"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


@dataclasses.dataclass(frozen=True)
class Rule:
    name: str
    params: typing.Sequence["Parameter"]
    body: "Value"


@dataclasses.dataclass(frozen=True)
class RuntimeErrorArithmeticError:
    msg: str


@dataclasses.dataclass(frozen=True)
class RuntimeErrorSerialization:
    msg: str


@dataclasses.dataclass(frozen=True)
class RuntimeErrorUnsupported:
    msg: str


@dataclasses.dataclass(frozen=True)
class RuntimeErrorTypeError:
    msg: str
    stack_trace: typing.Optional[str]


@dataclasses.dataclass(frozen=True)
class RuntimeErrorUnboundVariable:
    sym: str


@dataclasses.dataclass(frozen=True)
class RuntimeErrorStackOverflow:
    msg: str


@dataclasses.dataclass(frozen=True)
class RuntimeErrorQueryTimeout:
    msg: str


@dataclasses.dataclass(frozen=True)
class RuntimeErrorApplication:
    msg: str
    stack_trace: typing.Optional[str]


@dataclasses.dataclass(frozen=True)
class RuntimeErrorFileLoading:
    msg: str


class RuntimeError(Enum):
    content: typing.Union[
        "RuntimeErrorArithmeticError",
        "RuntimeErrorSerialization",
        "RuntimeErrorUnsupported",
        "RuntimeErrorTypeError",
        "RuntimeErrorUnboundVariable",
        "RuntimeErrorStackOverflow",
        "RuntimeErrorQueryTimeout",
        "RuntimeErrorApplication",
        "RuntimeErrorFileLoading",
    ]
    tag: typing.Literal[
        "ArithmeticError",
        "Serialization",
        "Unsupported",
        "TypeError",
        "UnboundVariable",
        "StackOverflow",
        "QueryTimeout",
        "Application",
        "FileLoading",
    ]
    _name: str = "RuntimeError"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag


@dataclasses.dataclass(frozen=True)
class Trace:
    node: "Node"
    children: typing.Sequence["Trace"]


@dataclasses.dataclass(frozen=True)
class TraceResult:
    trace: "Trace"
    formatted: str


ValueNumber = typing.NewType("ValueNumber", "Numeric")


ValueString = typing.NewType("ValueString", str)


ValueBoolean = typing.NewType("ValueBoolean", bool)


ValueExternalInstance = typing.NewType("ValueExternalInstance", "ExternalInstance")


ValueDictionary = typing.NewType("ValueDictionary", "Dictionary")


ValuePattern = typing.NewType("ValuePattern", "Pattern")


ValueCall = typing.NewType("ValueCall", "Call")


ValueList = typing.NewType("ValueList", typing.Sequence["Value"])


ValueVariable = typing.NewType("ValueVariable", str)


ValueRestVariable = typing.NewType("ValueRestVariable", str)


ValueExpression = typing.NewType("ValueExpression", "Operation")


ValuePartial = typing.NewType("ValuePartial", "Partial")


class Value(Enum):
    content: typing.Union[
        "ValueNumber",
        "ValueString",
        "ValueBoolean",
        "ValueExternalInstance",
        "ValueDictionary",
        "ValuePattern",
        "ValueCall",
        "ValueList",
        "ValueVariable",
        "ValueRestVariable",
        "ValueExpression",
        "ValuePartial",
    ]
    tag: typing.Literal[
        "Number",
        "String",
        "Boolean",
        "ExternalInstance",
        "Dictionary",
        "Pattern",
        "Call",
        "List",
        "Variable",
        "RestVariable",
        "Expression",
        "Partial",
    ]
    _name: str = "Value"

    def __init__(self, *, content, tag):
        self.content = content
        self.tag = tag
