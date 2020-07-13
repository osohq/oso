import json
from collections.abc import Iterable
from datetime import datetime, timedelta
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
    ffi_deserialize,
    ffi_serialize,
    load_str,
    check_result,
    is_null,
    new_id,
    manage_query,
    Predicate,
    to_c_str,
    Variable,
)


POLAR_TYPES = [int, float, bool, str, dict, type(None), list]


class QueryResult:
    """Response type of a call to the `query` API"""

    def __init__(self, results: list):
        self.success = len(results) > 0
        self.results = [r["bindings"] for r in results]
        self.traces = [r["trace"] for r in results]


# @TODO: Fix this! These need to be global for now so that `Oso.register_class`
# works from anywhere.
CLASSES = {}
CLASS_CONSTRUCTORS = {}


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

        # Register built-in classes.
        self.register_class(Http)
        self.register_class(PathMapper)
        self.register_class(datetime, name="Datetime")
        self.register_class(timedelta, name="Timedelta")

    def __del__(self):
        # Not usually needed but useful for tests since we make a lot of these.
        lib.polar_free(self.polar)

    ########## PUBLIC METHODS ##########

    def load_file(self, policy_file):
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

    def load_str(self, string):
        """Load a Polar string, checking that all inline queries succeed."""
        load_str(self.polar, string, None, self.__run_query)

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
            for res in self.__run_query(query):
                had_result = True
                print(f"Result: {res['bindings']}")
            if not had_result:
                print("False")

    def register_class(self, cls, *, name=None, from_polar=None):
        """Registers `cls` as a class accessible by Polar. `from_polar` can
        either be a method or a string. In the case of a string, Polar will
        look for the method using `getattr(cls, from_polar)`."""
        cls_name = cls.__name__ if name is None else name
        self.classes[cls_name] = cls
        self.class_constructors[cls_name] = from_polar
        self.register_constant(cls_name, cls)

    def register_constant(self, name, value):
        """Registers `value` as a Polar constant variable called `name`."""
        name = to_c_str(name)
        value = ffi_serialize(self._to_polar_term(value))
        lib.polar_register_constant(self.polar, name, value)

    ########## HIDDEN METHODS ##########

    def _load_queued_files(self):
        """Load queued policy files into the knowledge base."""
        while self.load_queue:
            filename = self.load_queue.pop(0)
            with open(filename) as file:
                load_str(self.polar, file.read(), filename, self.__run_query)

    def _query_str(self, string):
        self._load_queued_files()
        string = to_c_str(string)
        query = check_result(lib.polar_new_query(self.polar, string))
        for res in self.__run_query(query):
            yield res["bindings"]

    def _query_pred(self, query: Predicate, debug=False, single=False):
        """Query the knowledge base."""
        self._load_queued_files()
        query = ffi_serialize(self._to_polar_term(query))
        query = check_result(lib.polar_new_query_from_term(self.polar, query))
        results = []
        for res in self.__run_query(query):
            results.append(res)
            if single:
                break
        return QueryResult(results)

    def _to_python(self, v):
        """ Convert polar terms to python values """
        value = v["value"]
        tag = [*value][0]
        if tag in ["String", "Boolean"]:
            return value[tag]
        elif tag == "Number":
            return [*value[tag].values()][0]
        elif tag == "List":
            return [self._to_python(e) for e in value[tag]]
        elif tag == "Dictionary":
            return {k: self._to_python(v) for k, v in value[tag]["fields"].items()}
        elif tag == "ExternalInstance":
            return self.__get_instance(value[tag]["instance_id"])
        elif tag == "InstanceLiteral":
            # TODO(gj): Should InstanceLiterals ever be making it to Python?
            # convert instance literals to external instances
            cls_name = value[tag]["tag"]
            fields = value[tag]["fields"]["fields"]
            return self.__make_external_instance(cls_name, fields)
        elif tag == "Call":
            return Predicate(
                name=value[tag]["name"],
                args=[self._to_python(v) for v in value[tag]["args"]],
            )
        elif tag == "Variable":
            raise PolarRuntimeException(
                f"variable: {value} is unbound. make sure the value is set before using it in a method call"
            )
        raise PolarRuntimeException(f"cannot convert: {value} to Python")

    def _to_polar_term(self, v):
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
            val = {"List": [self._to_polar_term(i) for i in v]}
        elif type(v) == dict:
            val = {
                "Dictionary": {
                    "fields": {k: self._to_polar_term(v) for k, v in v.items()}
                }
            }
        elif isinstance(v, Predicate):
            val = {
                "Call": {
                    "name": v.name,
                    "args": [self._to_polar_term(v) for v in v.args],
                }
            }
        elif isinstance(v, Variable):
            # This is supported so that we can query for unbound variables
            val = {"Variable": v}
        else:
            val = {"ExternalInstance": {"instance_id": self.__cache_instance(v)}}
        term = {"value": val}
        return term

    ########## PRIVATE METHODS ##########

    def __run_query(self, q):
        """Method which performs the query loop over an already constructed query"""
        with manage_query(q) as query:
            while True:
                event_s = lib.polar_next_query_event(query)
                event = ffi_deserialize(event_s)
                if event == "Done":
                    break
                kind = [*event][0]
                data = event[kind]

                if kind == "MakeExternal":
                    self.__handle_make_external(data)
                if kind == "ExternalCall":
                    self.__handle_external_call(query, data)
                if kind == "ExternalOp":
                    self.__handle_external_op(query, data)
                if kind == "ExternalIsa":
                    self.__handle_external_isa(query, data)
                if kind == "ExternalUnify":
                    self.__handle_external_unify(query, data)
                if kind == "ExternalIsSubSpecializer":
                    self.__handle_external_is_subspecializer(query, data)
                if kind == "Debug":
                    self.__handle_debug(query, data)
                if kind == "Result":
                    bindings = {
                        k: self._to_python(v) for k, v in data["bindings"].items()
                    }
                    trace = data["trace"]
                    yield {"bindings": bindings, "trace": trace}

    def __make_external_instance(self, cls_name, fields, instance_id=None):
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
            fields = {k: self._to_python(v) for k, v in fields.items()}
            if constructor:
                instance = constructor(**fields)
            else:
                instance = cls(**fields)
            self.__cache_instance(instance, instance_id)
            return instance
        except Exception as e:
            raise PolarRuntimeException(
                f"Error constructing instance of {cls_name}: {e}"
            )

    def __cache_instance(self, instance, id=None):
        """Cache Python instance under externally generated id."""
        if id is None:
            id = new_id(self.polar)
        self.instances[id] = instance
        return id

    def __get_instance(self, id):
        """Look up Python instance by id."""
        if id not in self.instances:
            raise PolarRuntimeException(f"Unregistered instance: {id}.")
        return self.instances[id]

    def __handle_make_external(self, data):
        id = data["instance_id"]
        if id in self.instances:
            raise PolarRuntimeException(f"Instance {id} already registered.")
        cls_name = data["instance"]["tag"]
        fields = data["instance"]["fields"]["fields"]
        self.__make_external_instance(cls_name, fields, id)

    def __handle_external_call(self, query, data):
        call_id = data["call_id"]
        if call_id not in self.calls:
            value = data["instance"]["value"]
            if "ExternalInstance" in value:
                instance_id = value["ExternalInstance"]["instance_id"]
                instance = self.__get_instance(instance_id)
            else:
                instance = self._to_python(data["instance"])

            attribute = data["attribute"]
            args = [self._to_python(arg) for arg in data["args"]]

            # Lookup the attribute on the instance.
            try:
                attr = getattr(instance, attribute)
            except AttributeError:
                external_call(self.polar, query, call_id, None)
                # @TODO: polar line numbers in errors once polar errors are better.
                # raise PolarRuntimeException(f"Error calling {attribute}")
                return

            if callable(attr):  # If it's a function, call it with the args.
                result = attr(*args)
            else:  # If it's just an attribute, it's the result.
                result = attr

            # We now have either a generator or a result.
            # Call must be a generator so we turn anything else into one.
            if type(result) in POLAR_TYPES or not isinstance(result, Iterable):
                call = (i for i in [result])
            elif result is None:
                call = (_ for _ in [])
            else:
                call = iter(result)
            self.calls[call_id] = call

        # Return the next result of the call.
        try:
            value = next(self.calls[call_id])
            stringified = ffi_serialize(self._to_polar_term(value))
            external_call(self.polar, query, call_id, stringified)
        except StopIteration:
            external_call(self.polar, query, call_id, None)

    def __handle_external_op(self, query, data):
        op = data["operator"]
        args = [self._to_python(arg) for arg in data["args"]]
        answer: bool
        try:
            if op == "Lt":
                answer = args[0] < args[1]
            elif op == "Gt":
                answer = args[0] > args[1]
            elif op == "Eq":
                answer = args[0] == args[1]
            elif op == "Leq":
                answer = args[0] <= args[1]
            elif op == "Geq":
                answer = args[0] >= args[1]
            elif op == "Neq":
                answer = args[0] != args[1]
            else:
                raise PolarRuntimeException(
                    f"Unsupported external operation '{type(args[0])} {op} {type(args[1])}'"
                )
            external_answer(self.polar, query, data["call_id"], answer)
        except TypeError:
            raise PolarRuntimeException(
                f"External operation '{type(args[0])} {op} {type(args[1])}' failed."
            )

    def __handle_external_isa(self, query, data):
        cls_name = data["class_tag"]
        if cls_name in self.classes:
            instance = self.__get_instance(data["instance_id"])
            cls = self.classes[cls_name]
            isa = isinstance(instance, cls)
        else:
            isa = False
        external_answer(self.polar, query, data["call_id"], isa)

    def __handle_external_unify(self, query, data):
        left_instance_id = data["left_instance_id"]
        right_instance_id = data["right_instance_id"]
        try:
            left_instance = self.__get_instance(left_instance_id)
            right_instance = self.__get_instance(right_instance_id)
            eq = left_instance == right_instance
            external_answer(self.polar, query, data["call_id"], eq)
        except PolarRuntimeException:
            external_answer(self.polar, query, data["call_id"], False)

    def __handle_external_is_subspecializer(self, query, data):
        mro = self.__get_instance(data["instance_id"]).__class__.__mro__
        try:
            left = self.classes[data["left_class_tag"]]
            right = self.classes[data["right_class_tag"]]
            is_subspecializer = mro.index(left) < mro.index(right)
        except (KeyError, ValueError) as e:
            is_subspecializer = False
        finally:
            external_answer(self.polar, query, data["call_id"], is_subspecializer)

    def __handle_debug(self, query, data):
        if data["message"]:
            print(data["message"])
        command = input("> ")
        stringified = ffi_serialize(self._to_polar_term(command))
        check_result(lib.polar_debug_command(query, stringified))
