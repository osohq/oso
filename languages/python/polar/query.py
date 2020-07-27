from collections.abc import Iterable

from _polar_lib import lib
from .exceptions import PolarApiException
from .ffi import (
    external_answer,
    external_call,
    application_error,
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

    def __init__(self, ffi_query, *, host=None):
        self.ffi_query = ffi_query
        self.host = host
        self.calls = {}

    def __del__(self):
        del self.host
        lib.query_free(self.ffi_query)

    def run(self):
        """Run the event loop and yield results."""
        assert self.ffi_query, "no query to run"
        while True:
            event_s = lib.polar_next_query_event(self.ffi_query)
            event = ffi_deserialize(event_s)
            if event == "Done":
                break
            kind = [*event][0]
            data = event[kind]

            if kind == "MakeExternal":
                self.handle_make_external(data)
            if kind == "ExternalCall":
                self.handle_external_call(data)
            if kind == "ExternalOp":
                self.handle_external_op(data)
            if kind == "ExternalIsa":
                self.handle_external_isa(data)
            if kind == "ExternalUnify":
                self.handle_external_unify(data)
            if kind == "ExternalIsSubSpecializer":
                self.handle_external_is_subspecializer(data)
            if kind == "Debug":
                self.handle_debug(data)
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

    def handle_external_call(self, data):
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
            except AttributeError as e:
                application_error(self.ffi_query, str(e))
                external_call(self.ffi_query, call_id, None)
                return

            if callable(attr):  # If it's a function, call it with the args.
                result = attr(*args)
            else:  # If it's just an attribute, it's the result.
                result = attr

            # We now have either a generator or a result.
            # Call must be a generator so we turn anything else into one.
            if type(result) in NATIVE_TYPES or not isinstance(result, Iterable):
                call = (i for i in [result])
            else:
                call = iter(result)
            self.calls[call_id] = call

        # Return the next result of the call.
        try:
            value = next(self.calls[call_id])
            stringified = ffi_serialize(self.host.to_polar_term(value))
            external_call(self.ffi_query, call_id, stringified)
        except StopIteration:
            external_call(self.ffi_query, call_id, None)

    def handle_external_op(self, data):
        op = data["operator"]
        args = [self.host.to_python(arg) for arg in data["args"]]
        answer = self.host.operator(op, args)
        external_answer(self.ffi_query, data["call_id"], answer)

    def handle_external_isa(self, data):
        instance_id = data["instance_id"]
        class_tag = data["class_tag"]
        isa = self.host.isa(instance_id, class_tag)
        external_answer(self.ffi_query, data["call_id"], isa)

    def handle_external_unify(self, data):
        left_instance_id = data["left_instance_id"]
        right_instance_id = data["right_instance_id"]
        unify = self.host.unify(left_instance_id, right_instance_id)
        external_answer(self.ffi_query, data["call_id"], unify)

    def handle_external_is_subspecializer(self, data):
        instance_id = data["instance_id"]
        left_tag = data["left_class_tag"]
        right_tag = data["right_class_tag"]
        is_subspecializer = self.host.is_subspecializer(
            instance_id, left_tag, right_tag
        )
        external_answer(self.ffi_query, data["call_id"], is_subspecializer)

    def handle_debug(self, data):
        if data["message"]:
            print(data["message"])
        try:
            command = input("debug> ").strip(";")
        except EOFError:
            command = "continue"
        stringified = ffi_serialize(self.host.to_polar_term(command))
        check_result(lib.polar_debug_command(self.ffi_query, stringified))
