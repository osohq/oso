import json
from dataclasses import dataclass
from typing import Callable, List, Optional

from _polar_lib import ffi, lib

from .errors import get_python_error


@dataclass(frozen=True)
class PolarSource:
    src: str
    filename: Optional[str] = None


def read_c_str(c_str) -> str:
    """Copy a C string to a Python string and
    free the memory"""
    python_str = ffi.string(c_str).decode()
    lib.string_free(c_str)
    return python_str


class Polar:
    enrich_message: Callable
    """
    A method that can be called to enrich a debug, log, or error message from
    the core.
    """

    def __init__(self):
        self.ptr = lib.polar_new()

    def __del__(self):
        lib.polar_free(self.ptr)

    def new_id(self):
        """Request a unique ID from the canonical external ID tracker."""
        return lib.polar_get_external_id(self.ptr)

    def build_filter_plan(self, types, partial_results, variable, class_tag):
        """Get a filterplan for data filtering."""
        # @TODO(Steve): Pass types.
        typs = ffi_serialize(types)
        prs = ffi_serialize(partial_results)
        var = to_c_str(variable)
        class_tag = to_c_str(class_tag)
        plan = lib.polar_build_filter_plan(self.ptr, typs, prs, var, class_tag)
        process_messages(self.next_message)
        filter_plan_str = read_c_str(check_result(plan))
        filter_plan = json.loads(filter_plan_str)
        # @TODO(Steve): Decode Filter Plan to not just json?
        return filter_plan

    def build_data_filter(self, types, partial_results, variable, class_tag):
        """Get a filterplan for data filtering."""
        # @TODO(Steve): Pass types.
        typs = ffi_serialize(types)
        prs = ffi_serialize(partial_results)
        var = to_c_str(variable)
        class_tag = to_c_str(class_tag)
        plan = lib.polar_build_data_filter(self.ptr, typs, prs, var, class_tag)
        process_messages(self.next_message)
        filter_plan_str = read_c_str(check_result(plan))
        filter_plan = json.loads(filter_plan_str)
        # @TODO(Steve): Decode Filter Plan to not just json?
        return filter_plan

    def load(self, sources: List[PolarSource]):
        """Load Polar policies."""
        result = lib.polar_load(self.ptr, ffi_serialize([s.__dict__ for s in sources]))
        self.process_messages()
        self.check_result(result)

    def clear_rules(self):
        """Clear all rules from the Polar KB"""
        result = lib.polar_clear_rules(self.ptr)
        self.process_messages()
        self.check_result(result)

    def new_query_from_str(self, query_str):
        new_q_ptr = lib.polar_new_query(self.ptr, to_c_str(query_str), 0)
        self.process_messages()
        query = self.check_result(new_q_ptr)
        return Query(query)

    def new_query_from_term(self, query_term):
        new_q_ptr = lib.polar_new_query_from_term(
            self.ptr, ffi_serialize(query_term), 0
        )
        self.process_messages()
        query = self.check_result(new_q_ptr)
        return Query(query)

    def next_inline_query(self):
        q = lib.polar_next_inline_query(self.ptr, 0)
        self.process_messages()
        if is_null(q):
            return None
        return Query(q)

    def register_constant(self, value, name):
        name = to_c_str(name)
        value = ffi_serialize(value)
        result = lib.polar_register_constant(self.ptr, name, value)
        self.process_messages()
        self.check_result(result)

    def register_mro(self, name, mro):
        name = to_c_str(name)
        mro = ffi_serialize(mro)
        result = lib.polar_register_mro(self.ptr, name, mro)
        self.process_messages()
        self.check_result(result)

    def next_message(self):
        return lib.polar_next_polar_message(self.ptr)

    def set_message_enricher(self, enrich_message):
        self.enrich_message = enrich_message

    def check_result(self, result):
        return check_result(result, self.enrich_message)

    def process_messages(self):
        assert self.enrich_message, (
            "No message enricher on this instance of FfiPolar. You must call "
            "set_message_enricher before using process_messages."
        )
        for msg in process_messages(self.next_message):
            print(self.enrich_message(msg))


class Query:
    enrich_message: Callable
    """
    A method that can be called to enrich a debug, log, or error message from
    the core.
    """

    def __init__(self, ptr):
        self.ptr = ptr

    def __del__(self):
        lib.query_free(self.ptr)

    def call_result(self, call_id, value):
        """Make an external call and propagate FFI errors."""
        value = ffi_serialize(value)
        self.check_result(lib.polar_call_result(self.ptr, call_id, value))

    def question_result(self, call_id, answer):
        answer = 1 if answer else 0
        self.check_result(lib.polar_question_result(self.ptr, call_id, answer))

    def application_error(self, message):
        """Pass an error back to polar to get stack trace and other info."""
        message = to_c_str(message)
        self.check_result(lib.polar_application_error(self.ptr, message))

    def next_event(self) -> str:
        event = lib.polar_next_query_event(self.ptr)
        self.process_messages()
        event = read_c_str(self.check_result(event))
        return event

    def debug_command(self, command):
        result = lib.polar_debug_command(self.ptr, ffi_serialize(command))
        self.process_messages()
        self.check_result(result)

    def next_message(self):
        return lib.polar_next_query_message(self.ptr)

    def source(self):
        source = lib.polar_query_source_info(self.ptr)
        source = read_c_str(self.check_result(source))
        return source

    def bind(self, name, value):
        name = to_c_str(name)
        value = ffi_serialize(value)
        result = lib.polar_bind(self.ptr, name, value)
        # TODO(gj): Do we need to process_messages here?
        self.process_messages()
        self.check_result(result)

    def set_message_enricher(self, enrich_message):
        self.enrich_message = enrich_message

    def check_result(self, result):
        return check_result(result, self.enrich_message)

    def process_messages(self):
        assert self.enrich_message, (
            "No message enricher on this instance of FfiQuery. You must call "
            "set_message_enricher before using process_messages."
        )
        for msg in process_messages(self.next_message):
            print(self.enrich_message(msg))


def check_result(result, enrich_message=None):
    """
    Unwrap the result by (a) extracting the pointers for
    result and error, (b) freeing the result pointers, and then
    (c) either returning the result pointer, or constructing and
    raising the error.
    """
    # TODO: what type checking/assertions can we do here?

    r = result.result
    e = result.error
    # This is a bit of a hack. All CResult struct types
    # store two pointers: one to a result, one to an error
    # result_free doesn't care about what the pointers actually
    # point to though, so we can use the same result_free method
    lib.result_free(ffi.cast("polar_CResult_c_void *", result))
    if is_null(e):
        return r
    else:
        assert is_null(r), "internal error: result pointer must be null"
        error_str = read_c_str(e)
        error = get_python_error(error_str, enrich_message)
        raise error


def is_null(result):
    return result == ffi.NULL


def to_c_str(string):
    return ffi.new("char[]", string.encode())


def ffi_serialize(value):
    return to_c_str(json.dumps(value))


def process_messages(next_message_method):
    while True:
        msg_ptr = check_result(next_message_method())
        if is_null(msg_ptr):
            break
        msg_str = read_c_str(msg_ptr)
        message = json.loads(msg_str)

        kind = message["kind"]
        msg = message["msg"]

        if kind == "Print":
            yield msg
        elif kind == "Warning":
            yield f"[warning] {msg}"
