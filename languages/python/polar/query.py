from collections.abc import Iterable
import json

from _polar_lib import lib
from .exceptions import PolarApiException
from .ffi import Polar as FfiPolar, Query as FfiQuery
from .host import Host
from .predicate import Predicate
from .variable import Variable

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
        del self.ffi_query

    def run(self):
        """Run the event loop and yield results."""
        assert self.ffi_query, "no query to run"
        while True:
            ffi_event = self.ffi_query.next_event()
            event = json.loads(ffi_event.get())
            del ffi_event
            if event == "Done":
                break
            kind = [*event][0]
            data = event[kind]

            if kind == "MakeExternal":
                self.handle_make_external(data)
            elif kind == "ExternalCall":
                self.handle_external_call(data)
            elif kind == "ExternalOp":
                self.handle_external_op(data)
            elif kind == "ExternalIsa":
                self.handle_external_isa(data)
            elif kind == "ExternalUnify":
                self.handle_external_unify(data)
            elif kind == "ExternalIsSubSpecializer":
                self.handle_external_is_subspecializer(data)
            elif kind == "Debug":
                self.handle_debug(data)
            elif kind == "Result":
                bindings = {
                    k: self.host.to_python(v) for k, v in data["bindings"].items()
                }
                trace = data["trace"]
                yield {"bindings": bindings, "trace": trace}

    def handle_make_external(self, data):
        id = data["instance_id"]
        constructor = data["constructor"]["value"]
        if "InstanceLiteral" in constructor:
            cls_name = constructor["InstanceLiteral"]["tag"]
            fields = constructor["InstanceLiteral"]["fields"]["fields"]
            initargs = {k: self.host.to_python(v) for k, v in fields.items()}
        elif "Call" in constructor:
            cls_name = constructor["Call"]["name"]
            initargs = [self.host.to_python(arg) for arg in constructor["Call"]["args"]]
        else:
            raise PolarApiException("Bad constructor")
        self.host.make_instance(cls_name, initargs, id)

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
                self.ffi_query.application_error(str(e))
                self.ffi_query.call_result(call_id, None)
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
            self.ffi_query.call_result(call_id, self.host.to_polar_term(value))
        except StopIteration:
            self.ffi_query.call_result(call_id, None)

    def handle_external_op(self, data):
        op = data["operator"]
        args = [self.host.to_python(arg) for arg in data["args"]]
        answer = self.host.operator(op, args)
        self.ffi_query.question_result(data["call_id"], answer)

    def handle_external_isa(self, data):
        instance = data["instance"]
        class_tag = data["class_tag"]
        answer = self.host.isa(instance, class_tag)
        self.ffi_query.question_result(data["call_id"], answer)

    def handle_external_unify(self, data):
        left_instance_id = data["left_instance_id"]
        right_instance_id = data["right_instance_id"]
        answer = self.host.unify(left_instance_id, right_instance_id)
        self.ffi_query.question_result(data["call_id"], answer)

    def handle_external_is_subspecializer(self, data):
        instance_id = data["instance_id"]
        left_tag = data["left_class_tag"]
        right_tag = data["right_class_tag"]
        answer = self.host.is_subspecializer(instance_id, left_tag, right_tag)
        self.ffi_query.question_result(data["call_id"], answer)

    def handle_debug(self, data):
        if data["message"]:
            print(data["message"])
        try:
            command = input("debug> ").strip(";")
        except EOFError:
            command = "continue"
        self.ffi_query.debug_command(self.host.to_polar_term(command))
