"""Translate between Polar and the host language (Python)."""

from math import inf, isnan, nan
import re

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


# Entity classes


class OsoResource:
    pass


class OsoActor:
    pass


class OsoGroup:
    pass


class EntityMap:
    ENTITY_TYPES = {
        "OsoResource": OsoResource,
        "OsoActor": OsoActor,
        "OsoGroup": OsoGroup,
    }

    def __init__(self, cls_to_entity=None, entity_to_cls=None):
        self.cls_to_entity = (cls_to_entity or {}).copy()
        self.entity_to_cls = (entity_to_cls or {}).copy()

    def tag_is_entity(self, tag: str):
        return tag in EntityMap.ENTITY_TYPES.keys()

    def register(self, cls, entity_type: str):
        assert self.tag_is_entity(entity_type)
        self.cls_to_entity[cls] = self.ENTITY_TYPES[entity_type]
        self.entity_to_cls.setdefault(entity_type, []).append(cls)

    def get_type(self, cls):
        return self.cls_to_entity.get(cls)

    def instance_is_entity(self, python_instance, entity_tag):
        for cls in self.entity_to_cls.get(entity_tag):
            if isinstance(python_instance, cls):
                return True
        return False

    def class_is_entity(self, python_class, entity_tag):
        for cls in self.entity_to_cls.get(entity_tag):
            if issubclass(python_class, cls):
                return True
        return False

    def copy(self):
        return type(self)(
            cls_to_entity=self.cls_to_entity,
            entity_to_cls=self.entity_to_cls,
        )


class Host:
    """Maintain mappings and caches for Python classes & instances."""

    def __init__(
        self,
        polar,
        classes=None,
        class_ids=None,
        cls_names=None,
        instances=None,
        get_field=None,
        types=None,
        fetchers=None,
        entities=None,
    ):
        assert polar, "no Polar handle"
        self.ffi_polar = polar  # a "weak" handle, which we do not free
        self.classes = (classes or {}).copy()
        self.cls_names = (cls_names or {}).copy()
        self.class_ids = (
            class_ids or {}
        ).copy()  # Map from class name (Python) => instance ID used every time the class is converted to polar
        self.instances = (instances or {}).copy()
        self.types = (types or {}).copy()
        self.fetchers = (fetchers or {}).copy()
        self.entities = (entities or EntityMap()).copy()
        self._accept_expression = False  # default, see set_accept_expression

        # Check the types.
        def default_get_field(obj, field):
            return self.types_get_field(obj, field)

        self.get_field = get_field or default_get_field

    # @Q: I'm not really sure what I'm returning here.
    def types_get_field(self, obj, field):
        obj_type_name = self.cls_names[obj]
        if obj_type_name in self.types:
            obj_type_info = self.types[obj_type_name]
            if field in obj_type_info:
                field_type = obj_type_info[field]
                if field_type.kind == "parent":
                    return self.classes[field_type.other_type]
                elif field_type.kind == "children":
                    return list
            else:
                raise AttributeError(f"no field {field} on {obj.__name__}")
        raise PolarRuntimeError(f"No type information for Python class {obj.__name__}")

    def copy(self):
        """Copy an existing cache."""
        return type(self)(
            self.ffi_polar,
            classes=self.classes,
            cls_names=self.cls_names,
            class_ids=self.class_ids,
            instances=self.instances,
            get_field=self.get_field,
            types=self.types,
            fetchers=self.fetchers,
            entities=self.entities,
        )

    def get_class(self, name):
        """Fetch a Python class from the cache."""
        try:
            return self.classes[name]
        except KeyError:
            raise UnregisteredClassError(name)

    def cache_class(self, cls, entity_type, name=None):
        """Cache Python class by name."""
        name = cls.__name__ if name is None else name
        if name in self.classes.keys():
            raise DuplicateClassAliasError(name, self.get_class(name), cls)

        self.classes[name] = cls
        self.class_ids[cls] = self.cache_instance(cls)
        if entity_type:
            self.entities.register(cls, entity_type)
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
        if self.entities.tag_is_entity(class_tag):
            return self.entities.instance_is_entity(instance, class_tag)
        cls = self.get_class(class_tag)
        return isinstance(instance, cls)

    def isa_with_path(self, base_tag, path, class_tag) -> bool:
        base = self.get_class(base_tag)
        cls = self.get_class(class_tag)
        for field in path:
            field = self.to_python(field)
            base = self.get_field(base, field)
        if self.entities.tag_is_entity(class_tag):
            return self.entities.class_is_entity(base, class_tag)
        return issubclass(base, cls)

    def is_subclass(self, left_tag, right_tag) -> bool:
        """Return true if left is a subclass (or the same class) as right."""
        left = self.get_class(left_tag)
        right = self.get_class(right_tag)
        if self.entities.tag_is_entity(right_tag):
            return self.entities.class_is_entity(left, right_tag)
        return issubclass(left, right)

    def is_subspecializer(self, instance_id, left_tag, right_tag) -> bool:
        """Return true if the left class is more specific than the right class
        with respect to the given instance."""
        try:
            mro = self.get_instance(instance_id).__class__.__mro__
            left = self.get_class(left_tag)
            right = self.get_class(right_tag)
            # Base entity classes are never more specific
            if self.entities.tag_is_entity(right_tag):
                return True
            elif left_tag in EntityMap.ENTITY_TYPES:
                return False
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

    def enrich_message(self, message: str):
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
        elif isinstance(v, Predicate):
            val = {
                "Call": {
                    "name": v.name,
                    "args": [self.to_polar(v) for v in v.args],
                }
            }
        elif isinstance(v, Variable):
            val = {"Variable": v}
        elif isinstance(v, Expression):
            val = {
                "Expression": {
                    "operator": v.operator,
                    "args": [self.to_polar(v) for v in v.args],
                }
            }
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
        else:
            instance_id = None
            repr_str = None
            import inspect

            if inspect.isclass(v):
                instance_id = self.class_ids.get(v)
                # BEGIN HACK:
                # The polar core uses the .repr property to determine whether or not
                # to allow Roles.role_allows to be called with unbound variables as
                # arguments (only for sqlalchemy_oso)
                # Because of this, we need to continue to send the repr for
                # sqlalchemy_oso.roles.OsoRoles.Roles ONLY
                if (
                    "OsoRoles" in v.__qualname__
                    and v.__module__ == "sqlalchemy_oso.roles"
                ):
                    repr_str = repr(v)
                # END HACK
            val = {
                "ExternalInstance": {
                    "instance_id": self.cache_instance(v, instance_id),
                    "repr": repr_str,
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
