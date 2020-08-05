"""Translate between Polar and the host language (Python)."""

from .exceptions import PolarApiException, PolarRuntimeException
from .variable import Variable
from .predicate import Predicate


class Host:
    """Maintain mappings and caches for Python classes & instances."""

    def __init__(self, polar, classes={}, constructors={}, instances={}):
        assert polar, "no Polar handle"
        self.ffi_polar = polar  # a "weak" handle, which we do not free
        self.classes = classes.copy()
        self.constructors = constructors.copy()
        self.instances = instances.copy()

    def copy(self):
        """Copy an existing cache."""
        return type(self)(
            self.ffi_polar,
            classes=self.classes.copy(),
            constructors=self.constructors.copy(),
            instances=self.instances.copy(),
        )

    def get_class(self, name):
        """Fetch a Python class from the cache."""
        try:
            return self.classes[name]
        except KeyError:
            raise PolarRuntimeException(f"unregistered class {name}")

    def cache_class(self, cls, name=None, constructor=None):
        """Cache Python class by name."""
        if not isinstance(cls, type):
            raise PolarApiException(f"{cls} is not a class")
        name = cls.__name__ if name is None else name
        if not isinstance(name, str):
            raise PolarApiException(f"{name} is not a class name")

        self.classes[name] = cls
        self.constructors[name] = constructor or cls
        return name

    def get_constructor(self, name):
        """Fetch a constructor by name from the cache."""
        try:
            return self.constructors[name]
        except:
            raise PolarRuntimeException(f"missing constructor for class {name}")

    def get_instance(self, id):
        """Look up Python instance by id."""
        if id not in self.instances:
            raise PolarRuntimeException(f"unregistered instance {id}")
        return self.instances[id]

    def cache_instance(self, instance, id=None):
        """Cache Python instance under Polar-generated id."""
        if id is None:
            id = self.ffi_polar.new_id()
        self.instances[id] = instance
        return id

    def make_instance(self, name, initargs, id):
        """Make and cache a new instance of a Python class."""
        cls = self.get_class(name)
        constructor = self.get_constructor(name)
        if isinstance(constructor, str):
            constructor = getattr(cls, constructor)
        if id in self.instances:
            raise PolarRuntimeException(f"instance {id} is already registered")
        instance = (
            constructor(**initargs)
            if isinstance(initargs, dict)
            else constructor(*initargs)
        )
        self.cache_instance(instance, id)
        return instance

    def unify(self, left_instance_id, right_instance_id) -> bool:
        """Return true if the left instance is equal to the right."""
        try:
            left = self.get_instance(left_instance_id)
            right = self.get_instance(right_instance_id)
            return left == right
        except PolarRuntimeException:
            return False

    def isa(self, instance, class_tag) -> bool:
        try:
            instance = self.to_python(instance)
            cls = self.get_class(class_tag)
            return isinstance(instance, cls)
        except PolarRuntimeException:
            return False

    def is_subspecializer(self, instance_id, left_tag, right_tag) -> bool:
        """Return true if the left class is more specific than the right class
        with respect to the given instance."""
        try:
            mro = self.get_instance(instance_id).__class__.__mro__
            left = self.get_class(left_tag)
            right = self.get_class(right_tag)
            return mro.index(left) < mro.index(right)
        except (ValueError, PolarRuntimeException):
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
                raise PolarRuntimeException(
                    f"Unsupported external operation '{type(args[0])} {op} {type(args[1])}'"
                )
        except TypeError:
            raise PolarRuntimeException(
                f"External operation '{type(args[0])} {op} {type(args[1])}' failed."
            )

    def to_polar_term(self, v):
        """Convert a Python object to a Polar term."""
        if type(v) == bool:
            val = {"Boolean": v}
        elif type(v) == int:
            val = {"Number": {"Integer": v}}
        elif type(v) == float:
            val = {"Number": {"Float": v}}
        elif type(v) == str:
            val = {"String": v}
        elif type(v) == list:
            val = {"List": [self.to_polar_term(i) for i in v]}
        elif type(v) == dict:
            val = {
                "Dictionary": {
                    "fields": {k: self.to_polar_term(v) for k, v in v.items()}
                }
            }
        elif isinstance(v, Predicate):
            val = {
                "Call": {
                    "name": v.name,
                    "args": [self.to_polar_term(v) for v in v.args],
                }
            }
        elif isinstance(v, Variable):
            val = {"Variable": v}
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
            return [*value[tag].values()][0]
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

        raise PolarRuntimeException(f"cannot convert {value} to Python")
