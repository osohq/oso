"""Translate between Polar and the host language (Python)."""

from math import inf, isnan, nan

from .exceptions import (
    PolarApiError,
    PolarRuntimeError,
    UnregisteredClassError,
    DuplicateClassAliasError,
    UnregisteredInstanceError,
    DuplicateInstanceRegistrationError,
    UnexpectedPolarTypeError,
)
from .variable import Variable
from .predicate import Predicate
from .partial import Partial
from .expression import Expression, Pattern


class Host:
    """Maintain mappings and caches for Python classes & instances."""

    def __init__(self, polar, classes={}, instances={}):
        assert polar, "no Polar handle"
        self.ffi_polar = polar  # a "weak" handle, which we do not free
        self.classes = classes.copy()
        self.instances = instances.copy()

    def copy(self):
        """Copy an existing cache."""
        return type(self)(
            self.ffi_polar,
            classes=self.classes.copy(),
            instances=self.instances.copy(),
        )

    def get_class(self, name):
        """Fetch a Python class from the cache."""
        try:
            return self.classes[name]
        except KeyError:
            raise UnregisteredClassError(name)

    def cache_class(self, cls, name=None):
        """Cache Python class by name."""
        name = cls.__name__ if name is None else name
        if name in self.classes.keys():
            raise DuplicateClassAliasError(name, self.get_class(name), cls)

        self.classes[name] = cls
        return name

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
        except TypeError as e:
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
        elif isinstance(v, Predicate):
            val = {
                "Call": {
                    "name": v.name,
                    "args": [self.to_polar(v) for v in v.args],
                }
            }
        elif isinstance(v, Variable):
            val = {"Variable": v}
        elif isinstance(v, Partial):
            val = {"Partial": v.to_polar()}
        else:
            val = {
                "ExternalInstance": {
                    "instance_id": self.cache_instance(v),
                    "repr": repr(v),
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
