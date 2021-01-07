from collections.abc import Iterable
import json

from .exceptions import (
    InvalidIteratorError,
    InvalidCallError,
    InvalidConstructorError,
    PolarRuntimeError,
)
from .polar_types import (
    Call,
    QueryEventDebug,
    QueryEventDone,
    QueryEventExternalCall,
    QueryEventExternalIsSubSpecializer,
    QueryEventExternalIsSubclass,
    QueryEventExternalIsa,
    QueryEventExternalOp,
    QueryEventExternalUnify,
    QueryEventMakeExternal,
    QueryEventNextExternal,
    QueryEventResult,
    deserialize_json,
    QueryEvent,
)

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
            event = deserialize_json(ffi_event.get(), QueryEvent)
            del ffi_event

            call_map = {
                QueryEventMakeExternal: self.handle_make_external,
                QueryEventExternalCall: self.handle_external_call,
                QueryEventExternalOp: self.handle_external_op,
                QueryEventExternalIsa: self.handle_external_isa,
                QueryEventExternalUnify: self.handle_external_unify,
                QueryEventExternalIsSubSpecializer: self.handle_external_is_subspecializer,
                QueryEventExternalIsSubclass: self.handle_external_is_subclass,
                QueryEventNextExternal: self.handle_next_external,
                QueryEventDebug: self.handle_debug,
            }

            if isinstance(event, QueryEventDone):
                break
            elif isinstance(event, QueryEventResult):
                bindings = {
                    k: self.host.to_python(v) for k, v in event.bindings.items()
                }
                trace = event.trace
                yield {"bindings": bindings, "trace": trace}
            elif type(event) in call_map:
                call_map[type(event)](event)
            else:
                raise PolarRuntimeError(f"Unhandled event: {json.dumps(event)}")

    def handle_make_external(self, data: QueryEventMakeExternal):
        id = data.instance_id
        constructor = data.constructor
        if isinstance(constructor, Call):
            cls_name = constructor.name
            args = [self.host.to_python(arg) for arg in constructor.args]
            kwargs = constructor.kwargs or {}
            kwargs = {k: self.host.to_python(v) for k, v in kwargs.items()}
        else:
            raise InvalidConstructorError()
        self.host.make_instance(cls_name, args, kwargs, id)

    def handle_external_call(self, data: QueryEventExternalCall):
        call_id = data.call_id
        instance = self.host.to_python(data.instance)
        attribute = data.attribute

        # Lookup the attribute on the instance.
        try:
            attr = getattr(instance, attribute)
        except AttributeError as e:
            self.ffi_query.application_error(str(e))
            self.ffi_query.call_result(call_id, None)
            return
        if (
            callable(attr) and data.args is not None
        ):  # If it's a function, call it with the args.
            args = [self.host.to_python(arg) for arg in data.args]
            kwargs = data.kwargs or {}
            kwargs = {k: self.host.to_python(v) for k, v in kwargs.items()}
            result = attr(*args, **kwargs)
        elif data.args is not None:
            raise InvalidCallError(
                f"tried to call '{attribute}' but it is not callable"
            )
        else:  # If it's just an attribute, it's the result.
            result = attr

        # Return the result of the call.
        self.ffi_query.call_result(call_id, self.host.to_polar(result))

    def handle_external_op(self, data: QueryEventExternalOp):
        args = [self.host.to_python(arg) for arg in data.args]
        answer = self.host.operator(data.operator, args)
        self.ffi_query.question_result(data.call_id, answer)

    def handle_external_isa(self, data: QueryEventExternalIsa):
        answer = self.host.isa(data.instance, data.class_tag)
        self.ffi_query.question_result(data.call_id, answer)

    def handle_external_unify(self, data: QueryEventExternalUnify):
        answer = self.host.unify(data.left_instance_id, data.right_instance_id)
        self.ffi_query.question_result(data.call_id, answer)

    def handle_external_is_subspecializer(
        self, data: QueryEventExternalIsSubSpecializer
    ):
        answer = self.host.is_subspecializer(
            data.instance_id, data.left_class_tag, data.right_class_tag
        )
        self.ffi_query.question_result(data.call_id, answer)

    def handle_external_is_subclass(self, data: QueryEventExternalIsSubclass):
        answer = self.host.is_subclass(data.left_class_tag, data.right_class_tag)
        self.ffi_query.question_result(data.call_id, answer)

    def handle_next_external(self, data: QueryEventNextExternal):
        call_id = data.call_id
        iterable = data.iterable

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

    def handle_debug(self, data: QueryEventDebug):
        if data.message:
            print(data.message)
        try:
            command = input("debug> ").strip(";")
        except EOFError:
            command = "continue"
        self.ffi_query.debug_command(self.host.to_polar(command))
