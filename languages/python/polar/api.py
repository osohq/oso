import json

from _polar_lib import ffi, lib

from collections.abc import Iterable
from pathlib import Path
from types import GeneratorType
from typing import Any, Sequence
import weakref


from .exceptions import (
    InvalidTokenCharacter,
    Serialization,
    Unknown,
    ParserException,
    PolarApiException,
    PolarRuntimeException,
    IntegerOverflow,
    InvalidToken,
    InvalidTokenCharacter,
    UnrecognizedEOF,
    UnrecognizedToken,
    ExtraToken,
)


##### API Types ######

POLAR_TYPES = [int, float, bool, str, dict, type(None)]


class Query:
    """Request type for a `query` API call.

    :param name: the predicate to query
    :param args: a list of arguments to the predicate
    """

    name: str
    args: Sequence[Any]

    def __init__(self, name: str, args: Sequence[Any]):
        """Initialize Polar

        Initializes a new Polar FFI instance"""
        self.name = name
        self.args = args


class QueryResult:
    """Response type of a call to the `query` API"""

    def __init__(self, results: list):
        self.results = results
        self.success = len(results) > 0


#### Polar implementation

# These need to be global for now...
# So that register_python_class works from anywhere
# @TODO: Fix all examples to call polar.register_python_class and depreciate this.
CLASSES = {}
CLASS_CONSTRUCTORS = {}


def register_python_class(cls, from_polar=None):
    Polar().register_python_class(cls, from_polar)


class CleanupQuery:
    """ Context manager for the polar native query object. """

    def __init__(self, query):
        self.query = query

    def __enter__(self):
        return self.query

    def __exit__(self, type, value, traceback):
        lib.query_free(self.query)


class Polar:
    """Polar API"""

    def __init__(self):
        self.polar = lib.polar_new()
        self.loaded_files = {}
        self.load_queue = []
        global CLASSES
        self.classes = CLASSES
        global CLASS_CONSTRUCTORS
        self.class_constructors = CLASS_CONSTRUCTORS
        # set up the builtin isa rule
        self.id_to_instance = {}
        self.calls = {}
        self.load_str("isa(x, y, x: (y)); isa(x, y) := isa(x, y, x);")

    def __del__(self):
        # Not usually needed but useful for tests since we make a lot of these.
        lib.polar_free(self.polar)

    def load(self, policy_file):
        """Load in polar policies. By default, defers loading of knowledge base
        until a query is made.
        """

        policy_file = Path(policy_file)

        extension = policy_file.suffix
        if extension not in (".pol", ".polar"):
            raise PolarApiException(f"Policy names must have .pol or .polar extension")

        if not policy_file.exists():
            raise PolarApiException(f"Could not find file: {policy_file}")

        if policy_file not in self.load_queue:
            self.load_queue.append(policy_file)

    def _raise_error(self):
        # Raise polar errors as the correct python exception type.
        err_s = lib.polar_get_error()
        err_json = ffi.string(err_s).decode()
        error = json.loads(err_json)

        # All errors should be mapped to python exceptions.
        # Raise Unknown if we haven't mapped the error.
        exception = Unknown(f"Unknown Internal Error: {err_json}")

        kind = [*error][0]
        data = error[kind]

        if kind == "Parse":
            parse_err_kind = [*error][0]
            parse_err_data = error[kind]

            if parse_err_kind == "IntegerOverflow":
                token = parse_err_data["token"]
                pos = parse_err_data["pos"]
                exception = IntegerOverflow(token, pos)
            elif parse_err_kind == "InvalidTokenCharacter":
                token = parse_err_data["token"]
                c = parse_err_data["c"]
                pos = parse_err_data["pos"]
                exception = InvalidTokenCharacter(token, c, pos)
            elif parse_err_kind == "InvalidToken":
                pos = parse_err_data["pos"]
                exception = InvalidToken(pos)
            elif parse_err_kind == "UnrecognizedEOF":
                pos = parse_err_data["pos"]
                exception = UnrecognizedEOF(pos)
            elif parse_err_kind == "UnrecognizedToken":
                token = parse_err_data["token"]
                pos = parse_err_data["pos"]
                exception = UnrecognizedToken(token, pos)
            elif parse_err_kind == "ExtraToken":
                token = parse_err_data["token"]
                pos = parse_err_data["pos"]
                exception = ExtraToken(token, pos)
            else:
                exception = ParserException(f"Parser Exception: {json.dumps(data)}")

        elif kind == "Runtime":
            # @TODO: Runtime exception types.
            exception = PolarRuntimeException(json.dumps(data))

        elif kind == "Operational":
            if data == "Unknown":
                # This happens on panics from rust.
                exception = Unknown("Unknown Internal Error: See console.")

        lib.string_free(err_s)
        raise exception

    def _read_in_file(self, path):
        """Reads in a file and adds to the knowledge base."""
        with open(path) as file:
            # contents = ""
            # for line in file:
            #     line = line.split("#")[0]
            #     line = line.split("?=")[0]
            #     contents += line + "\n"
            contents = file.read()
            self.loaded_files[path] = contents
            self.load_str(contents)

    def _kb_load(self):
        """Load queued policy files into the knowledge base."""
        files = self.load_queue.copy()
        for policy_file in files:
            self._read_in_file(policy_file)
            self.load_queue.remove(policy_file)

    def import_builtin_module(self, name: str):
        raise NotImplementedError()

    def register_python_class(self, cls, from_polar=None):
        class_name = cls.__name__
        self.classes[class_name] = cls
        self.class_constructors[class_name] = from_polar

    def register_class(self, spec, source_class: type):
        raise NotImplementedError()

    def load_str(self, src_str):
        """Load string into knowledge base.

        If it contains inline queries, ensure they succeed."""
        c_str = ffi.new("char[]", src_str.encode())
        load = lib.polar_new_load(self.polar, c_str)
        if load == ffi.NULL:
            self._raise_error()

        try:
            while True:
                query = ffi.new("polar_Query **")
                loaded = lib.polar_load(self.polar, load, query)
                if loaded != 0:
                    self._raise_error()

                query = query[0]
                if query == ffi.NULL:
                    # Load is done
                    break

                results = self._do_query(query)
                try:
                    next(results)
                    # Exhaust generator to make sure query object is cleaned up before next load.
                    for _ in results:
                        pass

                except StopIteration:
                    # TODO (dhatch): Better error message.
                    raise PolarRuntimeException("Inline query in file failed.")
        finally:
            lib.load_free(load)

    def _to_external_id(self, python_obj):
        """ Create or look up a polar external_instance for an object """
        instance_id = lib.polar_get_external_id(self.polar)
        if instance_id == 0:
            self._raise_error()
        self.id_to_instance[instance_id] = python_obj
        return instance_id

    def _from_external_id(self, external_id):
        """ Lookup python object by external_id """
        assert external_id in self.id_to_instance
        return self.id_to_instance[external_id]

    def to_python(self, v):
        """ Convert polar terms to python values """
        # i = v['id']
        # offset = v['offset']
        value = v["value"]
        tag = [*value][0]
        if tag in ["Integer", "String", "Boolean"]:
            return value[tag]
        elif tag == "List":
            return [self.to_python(e) for e in value[tag]]
        elif tag == "Dictionary":
            return {k: self.to_python(v) for k, v in value[tag]["fields"].items()}
        elif tag == "ExternalInstance":
            return self._from_external_id(value[tag]["instance_id"])
        elif tag == "InstanceLiteral":
            # For instance literals, return the class type?
            cls_name = value[tag]["tag"]
            if cls_name in self.classes:
                return self.classes[cls_name]
            else:
                instance_id = lib.polar_get_external_id(self.polar)
                if instance_id == 0:
                    self._raise_error()
                id_to_instance[instance_id] = python_obj
                instance_to_id[python_obj] = instance_id
            return instance_id
        else:
            raise PolarRuntimeException(f"cannot convert: {value} to Python")

    def to_polar(self, v):
        """ Convert python values to polar terms """
        if type(v) == bool:
            val = {"Boolean": v}
        elif type(v) == int:
            val = {"Integer": v}
        elif type(v) == str:
            val = {"String": v}
        elif type(v) == list:
            val = {"List": [self.to_polar(i) for i in v]}
        elif type(v) == dict:
            val = {
                "Dictionary": {"fields": {k: self.to_polar(v) for k, v in v.items()}}
            }
        else:
            instance_id = self._to_external_id(v)
            val = {"ExternalInstance": {"instance_id": instance_id}}
        term = {"id": 0, "offset": 0, "value": val}
        return term

    def _do_query(self, q):
        """Method which performs the query loop over an already contructed query"""
        with CleanupQuery(q) as query:
            while True:
                event_s = lib.polar_query(self.polar, query)
                if event_s == ffi.NULL:
                    self._raise_error()

                event = json.loads(ffi.string(event_s).decode())
                lib.string_free(event_s)
                if event == "Done":
                    break

                kind = [*event][0]
                data = event[kind]

                if kind == "MakeExternal":
                    instance_id = data["instance_id"]
                    instance = data["instance"]

                    assert instance_id not in self.id_to_instance

                    class_name = instance["tag"]
                    term_fields = instance["fields"]["fields"]

                    fields = {}
                    for k, v in term_fields.items():
                        fields[k] = self.to_python(v)

                    if class_name not in self.classes:
                        raise PolarRuntimeException(
                            f"Error creating instance of class {class_name}. Has not been registered."
                        )

                    assert class_name in self.class_constructors

                    cls = self.classes[class_name]
                    constructor = self.class_constructors[class_name]
                    try:
                        if constructor:
                            instance = constructor(**fields)
                        else:
                            instance = cls(**fields)
                    except Exception as e:
                        raise PolarRuntimeException(
                            f"Error creating instance of class {class_name}. {e}"
                        )

                    self.id_to_instance[instance_id] = instance

                if kind == "ExternalCall":
                    call_id = data["call_id"]

                    if call_id not in self.calls:
                        # Create a new call if this is the first use of call_id.
                        instance_id = data["instance_id"]
                        attribute = data["attribute"]
                        args = [self.to_python(arg) for arg in data["args"]]

                        # Lookup the attribute on the instance.
                        instance = self.id_to_instance[instance_id]
                        try:
                            attr = getattr(instance, attribute)
                        except AttributeError:
                            # @TODO: polar line numbers in errors once polar errors are better.
                            raise PolarRuntimeException(f"Error calling {attribute}")

                        if callable(attr):
                            # If it's a function call it with the args.
                            result = attr(*args)
                        else:
                            # If it's just an attribute, it's the result.
                            result = attr

                        # We now have either a generator or a result.
                        # Call must be a generator so we turn anything else into one.
                        if type(result) in POLAR_TYPES or not isinstance(
                            result, GeneratorType
                        ):
                            call = (i for i in [result])
                        elif result is None:
                            call = (_ for _ in [])
                        else:
                            call = result

                        self.calls[call_id] = call

                    # Return the next result of the call.
                    try:
                        val = next(self.calls[call_id])
                        term = self.to_polar(val)
                        msg = json.dumps(term)
                        c_str = ffi.new("char[]", msg.encode())
                        result = lib.polar_external_call_result(
                            self.polar, query, call_id, c_str
                        )
                        if result == 0:
                            self._raise_error()
                    except StopIteration:
                        result = lib.polar_external_call_result(
                            self.polar, query, call_id, ffi.NULL
                        )
                        if result == 0:
                            self._raise_error()

                if kind == "ExternalIsa":
                    call_id = data["call_id"]
                    instance_id = data["instance_id"]
                    class_name = data["class_tag"]
                    instance = self.id_to_instance[instance_id]
                    # @TODO: make sure we even know about this class.
                    if class_name in self.classes:
                        cls = self.classes[class_name]
                        isa = isinstance(instance, cls)
                    else:
                        isa = False

                    result = lib.polar_external_question_result(
                        self.polar, query, call_id, 1 if isa else 0
                    )
                    if result == 0:
                        self._raise_error()

                if kind == "ExternalIsSubSpecializer":
                    call_id = data["call_id"]
                    instance_id = data["instance_id"]
                    left_class_tag = data["left_class_tag"]
                    right_class_tag = data["right_class_tag"]
                    instance = self.id_to_instance[instance_id]
                    instance_cls = instance.__class__
                    mro = instance_cls.__mro__
                    try:
                        left_class = self.classes[left_class_tag]
                        right_class = self.classes[right_class_tag]
                        is_sub_specializer = mro.index(left_class) < mro.index(
                            right_class
                        )
                    except (KeyError, ValueError) as e:
                        is_sub_specializer = False

                    result = lib.polar_external_question_result(
                        self.polar, query, call_id, 1 if is_sub_specializer else 0
                    )
                    if result == 0:
                        self._raise_error()

                if kind == "ExternalUnify":
                    call_id = data["call_id"]
                    left_instance_id = data["left_instance_id"]
                    right_instance_id = data["right_instance_id"]
                    left_instance = self.id_to_instance[left_instance_id]
                    right_instance = self.id_to_instance[right_instance_id]

                    # COMMENT (leina): does this get used? This isn't super useful behavior for instances because it only
                    # works predictably if they have `eq()` defined
                    unify = left_instance == right_instance

                    result = lib.polar_external_question_result(
                        self.polar, query, call_id, 1 if unify else 0
                    )
                    if result == 0:
                        self._raise_error()

                if kind == "Result":
                    yield {k: self.to_python(v) for k, v in data["bindings"].items()}

    def query_str(self, query_str):
        # Make sure KB is loaded in
        self._kb_load()
        self.clear_cache()

        c_str = ffi.new("char[]", query_str.encode())
        query = lib.polar_new_query(self.polar, c_str)
        if query == ffi.NULL:
            self._raise_error()

        yield from self._do_query(query)

    def query(self, query: Query, debug=False, single=False):
        """Query the knowledge base."""

        # Make sure KB is loaded in
        self._kb_load()
        self.clear_cache()

        query_term = json.dumps(
            {
                "id": 0,
                "offset": 0,
                "value": {
                    "Call": {
                        "name": query.name,
                        "args": [self.to_polar(arg) for arg in query.args],
                    }
                },
            }
        )
        c_str = ffi.new("char[]", query_term.encode())
        query = lib.polar_new_query_from_term(self.polar, c_str)
        if query == ffi.NULL:
            self._raise_error()

        results = []
        for res in self._do_query(query):
            results.append(res)
            if single:
                break

        result = QueryResult(results)
        # if result.success:
        #     log(query, result.results[0].trace())
        # else:
        #     log(query)
        return result

    def clear(self):
        """ Clear all facts and internal Polar classes from the knowledge base."""
        self.load_queue = []
        lib.polar_free(self.polar)
        self.polar = None
        self.polar = lib.polar_new()

    def clear_cache(self):
        self.id_to_instance = {}


class Http:
    """A resource accessed via HTTP."""

    def __init__(self, path="", query={}, hostname=None):
        raise NotImplementedError()

    def __repr__(self):
        raise NotImplementedError()

    def __str__(self):
        raise NotImplementedError()
