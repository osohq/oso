import json
from collections.abc import Iterable
from pathlib import Path
from types import GeneratorType
from typing import Any, Sequence, List

from _polar_lib import lib

from .errors import get_error
from .exceptions import PolarApiException, PolarRuntimeException
from .extras import Http, PathMapper
from .ffi import (
    external_answer,
    external_call,
    load_str,
    check_result,
    is_null,
    new_id,
    polar_query,
    Predicate,
    stringify,
    to_c_str,
    to_polar,
    unstringify,
    Variable,
)


POLAR_TYPES = [int, float, bool, str, dict, type(None), list]


class QueryResult:
    """Response type of a call to the `query` API"""

    def __init__(self, results: list):
        self.results = results
        self.success = len(results) > 0


# These need to be global for now...
# So that register_python_class works from anywhere
# @TODO: Fix all examples to call polar.register_python_class and depreciate this.
CLASSES = {}
CLASS_CONSTRUCTORS = {}


def register_python_class(cls, from_polar=None):
    Polar().register_python_class(cls, from_polar)


class Polar:
    """Polar API"""

    def __init__(self):
        self.polar = lib.polar_new()
        self.load_queue = []
        global CLASSES
        self.classes = CLASSES
        global CLASS_CONSTRUCTORS
        self.class_constructors = CLASS_CONSTRUCTORS
        self.instances = {}
        self.calls = {}
        # set up the builtin isa rule
        self.load_str("isa(x, y, _: (y)); isa(x, y) := isa(x, y, x);")

    def __del__(self):
        # Not usually needed but useful for tests since we make a lot of these.
        lib.polar_free(self.polar)

    def to_polar(self, value):
        return to_polar(value, self._cache_instance)

    def load(self, policy_file):
        """Load in polar policies. By default, defers loading of knowledge base
        until a query is made."""
        policy_file = Path(policy_file)
        extension = policy_file.suffix
        if extension not in (".pol", ".polar"):
            raise PolarApiException(f"Polar files must have .pol or .polar extension.")
        if not policy_file.exists():
            raise PolarApiException(f"Could not find file: {policy_file}")
        if policy_file not in self.load_queue:
            self.load_queue.append(policy_file)

    def _load_queued_files(self):
        """Load queued policy files into the knowledge base."""
        self._clear_instances()
        while self.load_queue:
            with open(self.load_queue.pop(0)) as file:
                self.load_str(file.read())

    def _clear_instances(self):
        self.instances = {}

    def register_python_class(self, cls, from_polar=None):
        """Registers `cls` as a class accessible by Polar.
        `from_polar` can either be a method or a string. In the case of a
        string, Polar will look for the method using `getattr(cls, from_polar)`"""

        class_name = cls.__name__
        self.classes[class_name] = cls
        self.class_constructors[class_name] = from_polar

    def load_str(self, string):
        """Load a Polar string, checking that all inline queries succeed."""
        load_str(self.polar, string, self._do_query)

    def _cache_instance(self, instance, id=None):
        """Cache Python instance under externally generated id."""
        if id is None:
            id = new_id(self.polar)
        self.instances[id] = instance
        return id

    def _lookup_instance(self, id):
        """Look up Python instance by id."""
        return self.instances[id]

    def to_python(self, v):
        """ Convert polar terms to python values """
        value = v["value"]
        tag = [*value][0]
        if tag in ["Integer", "String", "Boolean"]:
            return value[tag]
        elif tag == "List":
            return [self.to_python(e) for e in value[tag]]
        elif tag == "Dictionary":
            return {k: self.to_python(v) for k, v in value[tag]["fields"].items()}
        elif tag == "ExternalInstance":
            return self._lookup_instance(value[tag]["instance_id"])
        elif tag == "InstanceLiteral":
            # TODO(gj): Should InstanceLiterals ever be making it to Python?
            # convert instance literals to external instances
            cls_name = value[tag]["tag"]
            fields = value[tag]["fields"]["fields"]
            return self.make_external_instance(cls_name, fields)
        elif tag == "Call":
            return Predicate(
                name=value[tag]["name"],
                args=[self.to_python(v) for v in value[tag]["args"]],
            )
        elif tag == "Symbol":
            raise PolarRuntimeException(
                f"variable: {value} is unbound. make sure the value is set before using it in a method call"
            )
        raise PolarRuntimeException(f"cannot convert: {value} to Python")

    def make_external_instance(self, cls_name, fields, instance_id=None):
        """Make new instance of external class."""
        if cls_name not in self.classes:
            raise PolarRuntimeException(f"Unregistered class: {cls_name}.")
        if cls_name not in self.class_constructors:
            raise PolarRuntimeException(f"Missing constructor for class: {cls_name}.")
        cls = self.classes[cls_name]
        constructor = self.class_constructors[cls_name]
        try:
            # If constructor is a string, look it up on the class.
            if isinstance(constructor, str):
                constructor = getattr(cls, constructor)
            fields = {k: self.to_python(v) for k, v in fields.items()}
            if constructor:
                instance = constructor(**fields)
            else:
                instance = cls(**fields)
            self._cache_instance(instance, instance_id)
            return instance
        except Exception as e:
            raise PolarRuntimeException(
                f"Error constructing instance of {cls_name}: {e}"
            )

    def _do_query(self, q):
        """Method which performs the query loop over an already constructed query"""
        with polar_query(q) as query:
            while True:
                event_s = lib.polar_query(self.polar, query)
                event = unstringify(event_s)
                if event == "Done":
                    break

                kind = [*event][0]
                data = event[kind]

                if kind == "MakeExternal":
                    instance_id = data["instance_id"]
                    instance = data["instance"]

                    assert instance_id not in self.instances

                    class_name = instance["tag"]
                    term_fields = instance["fields"]["fields"]

                    self.make_external_instance(class_name, term_fields, instance_id)

                if kind == "ExternalCall":
                    call_id = data["call_id"]

                    if call_id not in self.calls:
                        # Create a new call if this is the first use of call_id.
                        instance_id = data["instance_id"]
                        attribute = data["attribute"]
                        args = [self.to_python(arg) for arg in data["args"]]

                        # Lookup the attribute on the instance.
                        instance = self.instances[instance_id]
                        try:
                            attr = getattr(instance, attribute)
                        except AttributeError:
                            external_call(self.polar, query, call_id, None)
                            continue
                            # @TODO: polar line numbers in errors once polar errors are better.
                            # raise PolarRuntimeException(f"Error calling {attribute}")

                        if callable(attr):
                            # If it's a function call it with the args.
                            result = attr(*args)
                        else:
                            # If it's just an attribute, it's the result.
                            result = attr

                        # We now have either a generator or a result.
                        # Call must be a generator so we turn anything else into one.
                        if type(result) in POLAR_TYPES or not isinstance(
                            result, Iterable
                        ):
                            call = (i for i in [result])
                        elif result is None:
                            call = (_ for _ in [])
                        else:
                            call = iter(result)

                        self.calls[call_id] = call

                    # Return the next result of the call.
                    try:
                        value = next(self.calls[call_id])
                        stringified = stringify(value, self._cache_instance)
                        external_call(self.polar, query, call_id, stringified)
                    except StopIteration:
                        external_call(self.polar, query, call_id, None)

                if kind == "ExternalIsa":
                    call_id = data["call_id"]
                    instance_id = data["instance_id"]
                    class_name = data["class_tag"]
                    instance = self.instances[instance_id]
                    # @TODO: make sure we even know about this class.
                    if class_name in self.classes:
                        cls = self.classes[class_name]
                        isa = isinstance(instance, cls)
                    else:
                        isa = False
                    external_answer(self.polar, query, call_id, isa)

                if kind == "ExternalIsSubSpecializer":
                    call_id = data["call_id"]
                    instance_id = data["instance_id"]
                    left_tag = data["left_class_tag"]
                    right_tag = data["right_class_tag"]
                    instance = self.instances[instance_id]
                    mro = instance.__class__.__mro__
                    try:
                        left = self.classes[left_tag]
                        right = self.classes[right_tag]
                        is_sub_specializer = mro.index(left) < mro.index(right)
                    except (KeyError, ValueError) as e:
                        is_sub_specializer = False
                    finally:
                        external_answer(self.polar, query, call_id, is_sub_specializer)

                if kind == "ExternalUnify":
                    call_id = data["call_id"]
                    left_id = data["left_instance_id"]
                    right_id = data["right_instance_id"]
                    left = self.instances[left_id]
                    right = self.instances[right_id]

                    # COMMENT (leina): does this get used? This isn't super
                    # useful behavior for instances because it only works
                    # predictably if they have `eq()` defined
                    unify = left == right
                    external_answer(self.polar, query, call_id, unify)

                if kind == "Debug":
                    if data["message"]:
                        print(data["message"])
                    command = input("> ")
                    # send input back across FFI
                    stringified = stringify(command, self._cache_instance)
                    check_result(
                        lib.polar_debug_command(self.polar, query, stringified)
                    )

                if kind == "Result":
                    yield {k: self.to_python(v) for k, v in data["bindings"].items()}

    def query_str(self, string):
        self._load_queued_files()
        string = to_c_str(string)
        query = check_result(lib.polar_new_query(self.polar, string))
        yield from self._do_query(query)

    def query(self, query: Predicate, debug=False, single=False):
        """Query the knowledge base."""
        self._load_queued_files()
        query = stringify(query, self._cache_instance)
        query = check_result(lib.polar_new_query_from_term(self.polar, query))
        results = []
        for res in self._do_query(query):
            results.append(res)
            if single:
                break
        return QueryResult(results)

    def clear(self):
        """Clear all facts and internal Polar classes from the knowledge base."""
        self.load_queue = []
        lib.polar_free(self.polar)
        self.polar = None
        self.polar = lib.polar_new()

    def repl(self):
        self._load_queued_files()
        while True:
            query = lib.polar_query_from_repl(self.polar)
            had_result = False
            if is_null(query):
                print("Query error: ", get_error())
                break
            for res in self._do_query(query):
                had_result = True
                print(f"Result: {res}")
            if not had_result:
                print("False")


register_python_class(Http)
register_python_class(PathMapper)
