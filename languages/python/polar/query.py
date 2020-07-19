from collections.abc import Iterable

from _polar_lib import lib
from .exceptions import PolarApiException
from .ffi import (
    external_answer,
    external_call,
    ffi_deserialize,
    ffi_serialize,
    load_str,
    check_result,
    is_null,
    new_id,
    Predicate,
    to_c_str,
    Variable,
)
from .host import Host

NATIVE_TYPES = [int, float, bool, str, dict, type(None), list]


class QueryResult:
    """Response type of a call to the `query` API"""

    def __init__(self, results: list):
        self.success = len(results) > 0
        self.results = [r["bindings"] for r in results]
        self.traces = [r["trace"] for r in results]


class Query:
    """Execute a Polar query through the FFI/event interface."""

    def __init__(self, polar, query=None, host=None, calls={}):
        if not polar:
            raise PolarApiException("no Polar handle")
        self.polar = polar
        self.query = query
        self.host = host.copy() if host else Host(polar)
        self.calls = calls.copy()

    def __del__(self):
        del self.host
        lib.query_free(self.query)

    def run(self, query=None):
        """Run the event loop and yield results."""
        if query is None:
            query = self.query
        else:
            self.query = query
        assert query, "no query to run"
        while True:
            event_s = lib.polar_next_query_event(query)
            event = ffi_deserialize(event_s)
            if event == "Done":
                break
            kind = [*event][0]
            data = event[kind]

            if kind == "MakeExternal":
                self.handle_make_external(data)
            if kind == "ExternalCall":
                self.handle_external_call(query, data)
            if kind == "ExternalOp":
                self.handle_external_op(query, data)
            if kind == "ExternalIsa":
                self.handle_external_isa(query, data)
            if kind == "ExternalUnify":
                self.handle_external_unify(query, data)
            if kind == "ExternalIsSubSpecializer":
                self.handle_external_is_subspecializer(query, data)
            if kind == "Debug":
                self.handle_debug(query, data)
            if kind == "Result":
                bindings = {
                    k: self.host.to_python(v) for k, v in data["bindings"].items()
                }
                trace = data["trace"]
                yield {"bindings": bindings, "trace": trace}

    def handle_make_external(self, data):
        id = data["instance_id"]
        cls_name = data["instance"]["tag"]
        fields = data["instance"]["fields"]["fields"]
        fields = {k: self.host.to_python(v) for k, v in fields.items()}
        self.host.make_instance(cls_name, fields, id)

    def handle_external_call(self, query, data):
        call_id = data["call_id"]
        if call_id not in self.calls:
            value = data["instance"]["value"]
            if "ExternalInstance" in value:
                instance_id = value["ExternalInstance"]["instance_id"]
                instance = self.host.get_instance(instance_id)
            else:
                instance = self.host.to_python(data["instance"])

            attribute = data["attribute"]
            args = [self.host.to_python(arg) for arg in data["args"]]

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
            if type(result) in NATIVE_TYPES or not isinstance(result, Iterable):
                call = (i for i in [result])
            elif result is None:
                call = (_ for _ in [])
            else:
                call = iter(result)
            self.calls[call_id] = call

        # Return the next result of the call.
        try:
            value = next(self.calls[call_id])
            stringified = ffi_serialize(self.host.to_polar_term(value))
            external_call(self.polar, query, call_id, stringified)
        except StopIteration:
            external_call(self.polar, query, call_id, None)

    def handle_external_op(self, query, data):
        op = data["operator"]
        args = [self.host.to_python(arg) for arg in data["args"]]
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

    def handle_external_isa(self, query, data):
        cls = self.host.get_class(data["class_tag"])
        if cls:
            instance = self.host.get_instance(data["instance_id"])
            isa = isinstance(instance, cls)
        else:
            isa = False
        external_answer(self.polar, query, data["call_id"], isa)

    def handle_external_unify(self, query, data):
        left_instance_id = data["left_instance_id"]
        right_instance_id = data["right_instance_id"]
        try:
            left_instance = self.host.get_instance(left_instance_id)
            right_instance = self.host.get_instance(right_instance_id)
            eq = left_instance == right_instance
            external_answer(self.polar, query, data["call_id"], eq)
        except PolarRuntimeException:
            external_answer(self.polar, query, data["call_id"], False)

    def handle_external_is_subspecializer(self, query, data):
        mro = self.host.get_instance(data["instance_id"]).__class__.__mro__
        left = self.host.get_class(data["left_class_tag"])
        right = self.host.get_class(data["right_class_tag"])
        try:
            is_subspecializer = mro.index(left) < mro.index(right)
        except ValueError:
            is_subspecializer = False
        external_answer(self.polar, query, data["call_id"], is_subspecializer)

    def handle_debug(self, query, data):
        if data["message"]:
            print(data["message"])
        command = input("> ")
        stringified = ffi_serialize(self.host.to_polar_term(command))
        check_result(lib.polar_debug_command(query, stringified))
