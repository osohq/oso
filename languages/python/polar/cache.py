from .exceptions import PolarApiException, PolarRuntimeException
from .ffi import new_id, Predicate, Variable


class Cache:
    """Maintain mappings and caches for Python classes & instances."""

    def __init__(self, polar, classes={}, constructors={}, instances={}):
        assert polar, "no Polar handle"
        self.polar = polar  # a "weak" handle, which we do not free
        self.classes = classes.copy()
        self.constructors = constructors.copy()
        self.instances = instances.copy()

    def copy(self):
        """Copy an existing cache."""
        return type(self)(
            self.polar,
            classes=self.classes.copy(),
            constructors=self.constructors.copy(),
            instances=self.instances.copy(),
        )

    def get_class(self, cls_name, default=None):
        return self.classes.get(cls_name, default)

    def cache_class(self, cls, cls_name, constructor=None):
        """Cache Python class by name."""
        if not isinstance(cls, type):
            raise PolarApiException(f"{cls} is not a class")
        if not isinstance(cls_name, str):
            raise PolarApiException(f"{cls_name} is not a class name")

        self.classes[cls_name] = cls
        self.constructors[cls_name] = constructor or cls

    def get_instance(self, id):
        """Look up Python instance by id."""
        if id not in self.instances:
            raise PolarRuntimeException(f"unregistered instance {id}")
        return self.instances[id]

    def cache_instance(self, instance, id=None):
        """Cache Python instance under Polar-generated id."""
        if id is None:
            id = new_id(self.polar)
        self.instances[id] = instance
        return id

    def make_instance(self, cls_name, fields, id):
        """Make and cache a new instance of a Python class."""
        cls = self.get_class(cls_name)
        if not cls:
            raise PolarRuntimeException(f"unregistered class {cls_name}")
        constructor = self.constructors.get(cls_name, cls)
        if not constructor:
            raise PolarRuntimeException(f"missing constructor for class {cls_name}")
        elif isinstance(constructor, str):
            constructor = getattr(cls, constructor)
        if id in self.instances:
            breakpoint()
            raise PolarRuntimeException(f"instance {id} is already registered")
        instance = constructor(**fields)
        self.cache_instance(instance, id)
        return instance

    def to_polar_term(self, v):
        """Convert Python values to Polar terms."""
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
            val = {"ExternalInstance": {"instance_id": self.cache_instance(v)}}
        term = {"value": val}
        return term

    def to_python(self, value):
        """ Convert polar terms to python values."""
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
            raise PolarRuntimeException(
                f"variable: {value} is unbound. make sure the value is set before using it in a method call"
            )
        raise PolarRuntimeException(f"cannot convert {value} to Python")
