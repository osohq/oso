"""Translate between Polar and the host language (Python)."""

from dataclasses import dataclass
from math import inf, isnan, nan
import re
import inspect
from typing import Any, Dict, Union


from .exceptions import (
    PolarRuntimeError,
    UnregisteredClassError,
    DuplicateClassAliasError,
    UnregisteredInstanceError,
    DuplicateInstanceRegistrationError,
    UnexpectedPolarTypeError,
    UNEXPECTED_EXPRESSION_MESSAGE,
)
from .variable import Variable
from .predicate import Predicate
from .expression import Expression, Pattern
from .data_filtering import Relation


@dataclass
class UserType:
    name: str
    cls: type
    id: int
    fields: Dict[str, Any]


class Host:
    """Maintain mappings and caches for Python classes & instances."""

    types: Dict[Union[str, type], UserType]

    def __init__(
        self,
        polar,
        types=None,
        instances=None,
        get_field=None,
        adapter=None,
    ):
        assert polar, "no Polar handle"
        self.ffi_polar = polar  # a "weak" handle, which we do not free
        # types maps class names (as string) and class objects to UserType.
        self.types = (types or {}).copy()
        self.instances = (instances or {}).copy()
        self._accept_expression = False  # default, see set_accept_expression
        self.adapter = adapter

        self.get_field = get_field or self.types_get_field

    # @Q: I'm not really sure what I'm returning here.
    def types_get_field(self, obj, field) -> type:
        if obj not in self.types:
            raise PolarRuntimeError(
                f"No type information for Python class {obj.__name__}"
            )
        rec = self.types[obj]

        if field not in rec.fields:
            raise PolarRuntimeError(f"No field {field} on {obj.__name__}")
        field_type = rec.fields[field]

        if not isinstance(field_type, Relation):
            return field_type

        if field_type.kind == "one":
            return self.types[field_type.other_type].cls
        elif field_type.kind == "many":
            return list
        else:
            raise PolarRuntimeError(f"Invalid kind {field_type.kind}")

    def copy(self) -> "Host":
        """Copy an existing cache."""
        return type(self)(
            self.ffi_polar,
            types=self.types,
            instances=self.instances,
            get_field=self.get_field,
            adapter=self.adapter,
        )

    def get_class(self, name):
        """Fetch a Python class from the cache."""
        try:
            return self.types[name].cls
        except KeyError:
            raise UnregisteredClassError(name)

    def distinct_user_types(self):
        return map(
            lambda k: self.types[k],
            filter(lambda k: isinstance(k, str), self.types.keys()),
        )

    def cache_class(
        self,
        cls,
        name=None,
        fields=None,
    ):
        """Cache Python class by name."""
        name = cls.__name__ if name is None else name
        if name in self.types.keys():
            raise DuplicateClassAliasError(name, self.get_class(name), cls)

        self.types[name] = self.types[cls] = UserType(
            name=name,
            cls=cls,
            id=self.cache_instance(cls),
            fields=fields or {},
        )
        return name

    def register_mros(self) -> None:
        """Register the MRO of each registered class to be used for rule type validation."""
        # Get MRO of all registered classes
        for rec in self.distinct_user_types():
            mro = [self.types[c].id for c in inspect.getmro(rec.cls) if c in self.types]
            self.ffi_polar.register_mro(rec.name, mro)

    def get_instance(self, id):
        """Look up Python instance by id."""
        if id not in self.instances:
            raise UnregisteredInstanceError(id)
        return self.instances[id]

    def cache_instance(self, instance, id=None):
        """Cache Python instance under Polar-generated id."""
        if id is None:
            id = self.ffi_polar.new_id()
        self.instances[id] = instance
        return id

    def make_instance(self, name, args, kwargs, id):
        """Construct and cache a Python instance."""
        if id in self.instances:
            raise DuplicateInstanceRegistrationError(id)
        cls = self.get_class(name)
        try:
            instance = cls(*args, **kwargs)
        except Exception as e:
            raise PolarRuntimeError(f"Error constructing instance of {name}: {e}")
        return self.cache_instance(instance, id)

    def unify(self, left_instance_id, right_instance_id) -> bool:
        """Return true if the left instance is equal to the right."""
        left = self.get_instance(left_instance_id)
        right = self.get_instance(right_instance_id)
        return left == right

    def isa(self, instance, class_tag) -> bool:
        instance = self.to_python(instance)
        cls = self.get_class(class_tag)
        return isinstance(instance, cls)

    def isa_with_path(self, base_tag, path, class_tag) -> bool:
        base = self.get_class(base_tag)
        cls = self.get_class(class_tag)
        for field in path:
            field = self.to_python(field)
            base = self.get_field(base, field)
        return issubclass(base, cls)

    def is_subclass(self, left_tag, right_tag) -> bool:
        """Return true if left is a subclass (or the same class) as right."""
        left = self.get_class(left_tag)
        right = self.get_class(right_tag)
        return issubclass(left, right)

    def is_subspecializer(self, instance_id, left_tag, right_tag) -> bool:
        """Return true if the left class is more specific than the right class
        with respect to the given instance."""
        try:
            mro = self.get_instance(instance_id).__class__.__mro__
            left = self.get_class(left_tag)
            right = self.get_class(right_tag)
            return mro.index(left) < mro.index(right)
        except ValueError:
            return False

    def operator(self, op, args):
        try:
            if op == "Lt":
                return args[0] < args[1]
            elif op == "Gt":
                return args[0] > args[1]
            elif op == "Eq":
                return args[0] == args[1]
            elif op == "Leq":
                return args[0] <= args[1]
            elif op == "Geq":
                return args[0] >= args[1]
            elif op == "Neq":
                return args[0] != args[1]
            else:
                raise PolarRuntimeError(
                    f"Unsupported external operation '{type(args[0])} {op} {type(args[1])}'"
                )
        except TypeError:
            raise PolarRuntimeError(
                f"External operation '{type(args[0])} {op} {type(args[1])}' failed."
            )

    def enrich_message(self, message: str) -> str:
        """
        "Enrich" a message from the polar core, such as a log line, debug
        message, or error trace.

        Currently only used to enrich messages with instance reprs. This allows
        us to avoid sending reprs eagerly when an instance is created in polar.
        """

        def replace_repr(match):
            instance_id = int(match[1])
            try:
                instance = self.get_instance(instance_id)
                return repr(instance)
            except UnregisteredInstanceError:
                return match[0]

        return re.sub(r"\^\{id: ([0-9]+)\}", replace_repr, message, flags=re.M)

    def to_polar(self, v):
        """Convert a Python object to a Polar term."""
        if type(v) == bool:
            val = {"Boolean": v}
        elif type(v) == int:
            val = {"Number": {"Integer": v}}
        elif type(v) == float:
            if v == inf:
                v = "Infinity"
            elif v == -inf:
                v = "-Infinity"
            elif isnan(v):
                v = "NaN"
            val = {"Number": {"Float": v}}
        elif type(v) == str:
            val = {"String": v}
        elif type(v) == list:
            val = {"List": [self.to_polar(i) for i in v]}
        elif type(v) == dict:
            val = {
                "Dictionary": {"fields": {k: self.to_polar(v) for k, v in v.items()}}
            }
        # only used when you call oso.query() with a Predicate instance
        elif isinstance(v, Predicate):
            val = {
                "Call": {
                    "name": v.name,
                    "args": [self.to_polar(v) for v in v.args],
                }
            }
        # basically only used in data filtering or if someone intentionally manually passes in a Variable instance
        elif isinstance(v, Variable):
            val = {"Variable": v}
        # basically only used in data filtering
        elif isinstance(v, Expression):
            val = {
                "Expression": {
                    "operator": v.operator,
                    "args": [self.to_polar(v) for v in v.args],
                }
            }
        # basically only used in data filtering (seeding the authorized_query()
        # call with an initial type binding so we know what type of resources
        # we're trying to determine access for)
        elif isinstance(v, Pattern):
            if v.tag is None:
                val = {"Pattern": self.to_polar(v.fields)["value"]}
            else:
                val = {
                    "Pattern": {
                        "Instance": {
                            "tag": v.tag,
                            "fields": self.to_polar(v.fields)["value"]["Dictionary"],
                        }
                    }
                }

        # user queries: oso.allow(<some user instance>, "some action", <some resource instance>)
        # Host.to_polar translates that into something like
        #   Call {
        #       name: String("allow"),
        #       args: List([
        #           ExternalInstance { instance_id: 1, repr: "<some user instance>", class_id: <user_class_id> },
        #           String("some action"),
        #           ExternalInstance { instance_id: 2, repr: "<some resource instance>", class_id: <resource_class_id> },
        #       ]
        #   }
        else:
            import inspect

            instance_id = None
            class_id = None

            # maintain consistent IDs for registered classes
            if inspect.isclass(v):
                if v in self.types:
                    class_id = instance_id = self.types[v].id

            # pass the class_repr only for registered types otherwise None
            class_repr = self.types[type(v)].name if type(v) in self.types else None

            # pass class_id for classes & instances of registered classes,
            # otherwise pass None
            if type(v) in self.types:
                class_id = self.types[type(v)].id

            val = {
                "ExternalInstance": {
                    "instance_id": self.cache_instance(v, instance_id),
                    "repr": None,
                    "class_repr": class_repr,
                    "class_id": class_id,
                }
            }
        term = {"value": val}
        return term

    def to_python(self, value):
        """Convert a Polar term to a Python object."""
        value = value["value"]
        tag = [*value][0]
        if tag in ["String", "Boolean"]:
            return value[tag]
        elif tag == "Number":
            number = [*value[tag].values()][0]
            if "Float" in value[tag]:
                if number == "Infinity":
                    return inf
                elif number == "-Infinity":
                    return -inf
                elif number == "NaN":
                    return nan
                else:
                    if not isinstance(number, float):
                        raise PolarRuntimeError(
                            f'Expected a floating point number, got "{number}"'
                        )
            return number
        elif tag == "List":
            return [self.to_python(e) for e in value[tag]]
        elif tag == "Dictionary":
            return {k: self.to_python(v) for k, v in value[tag]["fields"].items()}
        elif tag == "ExternalInstance":
            return self.get_instance(value[tag]["instance_id"])
        elif tag == "Call":
            return Predicate(
                name=value[tag]["name"],
                args=[self.to_python(v) for v in value[tag]["args"]],
            )
        elif tag == "Variable":
            return Variable(value[tag])
        elif tag == "Expression":
            if not self._accept_expression:
                raise UnexpectedPolarTypeError(UNEXPECTED_EXPRESSION_MESSAGE)

            args = list(map(self.to_python, value[tag]["args"]))
            operator = value[tag]["operator"]

            return Expression(operator, args)
        elif tag == "Pattern":
            pattern_tag = [*value[tag]][0]
            if pattern_tag == "Instance":
                instance = value[tag]["Instance"]
                return Pattern(instance["tag"], instance["fields"]["fields"])
            elif pattern_tag == "Dictionary":
                dictionary = value[tag]["Dictionary"]
                return Pattern(None, dictionary["fields"])
            else:
                raise UnexpectedPolarTypeError("Pattern: " + value[tag])

        raise UnexpectedPolarTypeError(tag)

    def set_accept_expression(self, accept):
        """Set whether the Host accepts Expression types from Polar, or raises an error."""
        self._accept_expression = accept
