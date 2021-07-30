import json
from typing import Callable

from _polar_lib import ffi, lib

from .errors import get_python_error


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
        return self.check_result(lib.polar_get_external_id(self.ptr))

    def enable_roles(self):
        """Load the built-in roles policy."""
        result = lib.polar_enable_roles(self.ptr)
        self.process_messages()
        self.check_result(result)

    def validate_roles_config(self, config_data):
        """Validate the user's Oso Roles config."""
        string = ffi_serialize(config_data)
        result = lib.polar_validate_roles_config(self.ptr, string)
        self.process_messages()
        self.check_result(result)

    def load(self, string, filename=None):
        """Load a Polar string, checking that all inline queries succeed."""
        string = to_c_str(string)
        filename = to_c_str(str(filename)) if filename else ffi.NULL
        result = lib.polar_load(self.ptr, string, filename)
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
        if value is None:
            value = ffi.NULL
        else:
            value = ffi_serialize(value)
        self.check_result(lib.polar_call_result(self.ptr, call_id, value))

    def question_result(self, call_id, answer):
        answer = 1 if answer else 0
        self.check_result(lib.polar_question_result(self.ptr, call_id, answer))

    def application_error(self, message):
        """Pass an error back to polar to get stack trace and other info."""
        message = to_c_str(message)
        self.check_result(lib.polar_application_error(self.ptr, message))

    def next_event(self):
        event = lib.polar_next_query_event(self.ptr)
        self.process_messages()
        event = self.check_result(event)
        return QueryEvent(event)

    def debug_command(self, command):
        result = lib.polar_debug_command(self.ptr, ffi_serialize(command))
        self.process_messages()
        self.check_result(result)

    def next_message(self):
        return lib.polar_next_query_message(self.ptr)

    def source(self):
        source = lib.polar_query_source_info(self.ptr)
        source = self.check_result(source)
        return Source(source)

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


class QueryEvent:
    def __init__(self, ptr):
        self.ptr = ptr

    def get(self):
        return ffi.string(self.ptr).decode()

    def __del__(self):
        lib.string_free(self.ptr)


class Error:
    def __init__(self):
        self.ptr = lib.polar_get_error()

    def get(self, enrich_message=None):
        return get_python_error(ffi.string(self.ptr).decode(), enrich_message)

    def __del__(self):
        lib.string_free(self.ptr)


class Source:
    def __init__(self, ptr):
        self.ptr = ptr

    def get(self):
        return ffi.string(self.ptr).decode()

    def __del__(self):
        lib.string_free(self.ptr)


def check_result(result, enrich_message=None):
    if result == 0 or is_null(result):
        raise Error().get(enrich_message)
    return result


def is_null(result):
    return result == ffi.NULL


def to_c_str(string):
    return ffi.new("char[]", string.encode())


def ffi_serialize(value):
    return to_c_str(json.dumps(value))


def process_messages(next_message_method):
    while True:
        msg_ptr = next_message_method()
        if is_null(msg_ptr):
            break
        msg_str = ffi.string(msg_ptr).decode()
        lib.string_free(msg_ptr)
        message = json.loads(msg_str)

        kind = message["kind"]
        msg = message["msg"]

        if kind == "Print":
            yield msg
        elif kind == "Warning":
            yield f"[warning] {msg}"
