from collections.abc import Iterable
import json

from .exceptions import (
    InvalidIteratorError,
    InvalidCallError,
    InvalidConstructorError,
    PolarRuntimeError,
)
from .data_filtering import Relation, Filter

NATIVE_TYPES = [int, float, bool, str, dict, type(None), list]


class QueryResult:
    """Response type of a call to the `query` API"""

    def __init__(self, results: list):
        self.success = len(results) > 0
        self.results = [r["bindings"] for r in results]
        self.traces = [r["trace"] for r in results]


class Query:
    """Execute a Polar query through the FFI/event interface."""

    def __init__(self, ffi_query, *, host=None, bindings=None):
        self.ffi_query = ffi_query
        self.ffi_query.set_message_enricher(host.enrich_message)
        self.host = host
        self.calls = {}
        for (k, v) in (bindings or {}).items():
            self.bind(k, v)

    def __del__(self):
        del self.host
        del self.ffi_query

    def bind(self, name, value):
        """Bind `name` to `value` for the duration of the query."""
        self.ffi_query.bind(name, self.host.to_polar(value))

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
                "ExternalIsaWithPath": self.handle_external_isa_with_path,
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
            # Check if it's a relationship
            attr = None
            cls = instance.__class__
            if cls in self.host.types:
                cls_rec = self.host.types[cls]
                typ = cls_rec.fields
                if attribute in typ:
                    attr_typ = typ[attribute]
                    if isinstance(attr_typ, Relation):
                        rel = attr_typ
                        # Use the fetcher for the other type to traverse the relationship
                        build_query = self.host.types[rel.other_type].build_query
                        exec_query = self.host.types[rel.other_type].exec_query
                        assert build_query is not None
                        assert exec_query is not None
                        constraint = Filter(
                            kind="Eq",
                            field=rel.other_field,
                            value=getattr(instance, rel.my_field),
                        )
                        constraints = [constraint]
                        query = build_query(constraints)
                        results = exec_query(query)
                        if rel.kind == "one":
                            assert len(results) == 1
                            attr = results[0]
                        elif rel.kind == "many":
                            attr = results
            if attr is None:
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

    def handle_external_isa_with_path(self, data):
        base_tag = data["base_tag"]
        path = data["path"]
        class_tag = data["class_tag"]
        try:
            answer = self.host.isa_with_path(base_tag, path, class_tag)
            self.ffi_query.question_result(data["call_id"], answer)
        except AttributeError as e:
            # TODO(gj): make sure we are printing but not failing on receipt of
            # this error in core.
            self.ffi_query.application_error(str(e))
            self.ffi_query.question_result(data["call_id"], False)

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
            print(self.host.enrich_message(data["message"]))
        try:
            command = input("debug> ").strip(";")
        except EOFError:
            command = "continue"
        self.ffi_query.debug_command(self.host.to_polar(command))
