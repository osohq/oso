from collections.abc import Iterable
import json

from _polar_lib import lib
from .exceptions import (
    InvalidIteratorError,
    PolarApiError,
    InvalidCallError,
    InvalidConstructorError,
    PolarRuntimeError,
)
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
            kind = [*event][0]
            data = event[kind]

            call_map = {
                "MakeExternal": self.handle_make_external,
                "ExternalCall": self.handle_external_call,
                "ExternalOp": self.handle_external_op,
                "ExternalIsa": self.handle_external_isa,
                "ExternalUnify": self.handle_external_unify,
                "ExternalIsSubSpecializer": self.handle_external_is_subspecializer,
                "ExternalIsSubclass": self.handle_external_is_subclass,
                "NextExternal": self.handle_next_external,
                "Debug": self.handle_debug,
            }

            if kind == "Done":
                break
            elif kind == "Result":
                bindings = {
                    k: self.host.to_python(v) for k, v in data["bindings"].items()
                }
                trace = data["trace"]
                yield {"bindings": bindings, "trace": trace}
            elif kind in call_map:
                call_map[kind](data)
            else:
                raise PolarRuntimeError(f"Unhandled event: {json.dumps(event)}")

    def handle_make_external(self, data):
        id = data["instance_id"]
        constructor = data["constructor"]["value"]
        if "Call" in constructor:
            cls_name = constructor["Call"]["name"]
            args = [self.host.to_python(arg) for arg in constructor["Call"]["args"]]
            kwargs = constructor["Call"]["kwargs"] or {}
            kwargs = {k: self.host.to_python(v) for k, v in kwargs.items()}
        else:
            raise InvalidConstructorError()
        self.host.make_instance(cls_name, args, kwargs, id)

    def handle_external_call(self, data):
        call_id = data["call_id"]
        instance = self.host.to_python(data["instance"])

        attribute = data["attribute"]

        # Lookup the attribute on the instance.
        try:
            attr = getattr(instance, attribute)
        except AttributeError as e:
            self.ffi_query.application_error(str(e))
            self.ffi_query.call_result(call_id, None)
            return
        if (
            callable(attr) and not data["args"] is None
        ):  # If it's a function, call it with the args.
            args = [self.host.to_python(arg) for arg in data["args"]]
            kwargs = data["kwargs"] or {}
            kwargs = {k: self.host.to_python(v) for k, v in kwargs.items()}
            result = attr(*args, **kwargs)
        elif not data["args"] is None:
            raise InvalidCallError(
                f"tried to call '{attribute}' but it is not callable"
            )
        else:  # If it's just an attribute, it's the result.
            result = attr

        # Return the result of the call.
        self.ffi_query.call_result(call_id, self.host.to_polar(result))

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

    def handle_external_is_subclass(self, data):
        left_tag = data["left_class_tag"]
        right_tag = data["right_class_tag"]
        answer = self.host.is_subclass(left_tag, right_tag)
        self.ffi_query.question_result(data["call_id"], answer)

    def handle_next_external(self, data):
        call_id = data["call_id"]
        iterable = data["iterable"]

        if call_id not in self.calls:
            value = self.host.to_python(iterable)
            if isinstance(value, Iterable):
                self.calls[call_id] = iter(value)
            else:
                raise InvalidIteratorError(f"{value} is not iterable")

        # Return the next result of the call.
        try:
            value = next(self.calls[call_id])
            self.ffi_query.call_result(call_id, self.host.to_polar(value))
        except StopIteration:
            self.ffi_query.call_result(call_id, None)

    def handle_debug(self, data):
        if data["message"]:
            print(data["message"])
        try:
            command = input("debug> ").strip(";")
        except EOFError:
            command = "continue"
        self.ffi_query.debug_command(self.host.to_polar(command))
