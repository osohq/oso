from contextlib import contextmanager
from dataclasses import dataclass
import json

from _polar_lib import ffi, lib

from .errors import get_python_error
from .exceptions import PolarRuntimeError


class Polar:
    def __init__(self):
        self.ptr = lib.polar_new()

    def __del__(self):
        lib.polar_free(self.ptr)

    def new_id(self):
        """Request a unique ID from the canonical external ID tracker."""
        return check_result(lib.polar_get_external_id(self.ptr))

    def load(self, string, filename=None):
        """Load a Polar string, checking that all inline queries succeed."""
        string = to_c_str(string)
        filename = to_c_str(str(filename)) if filename else ffi.NULL
        result = lib.polar_load(self.ptr, string, filename)
        process_messages(self.next_message)
        check_result(result)

    def clear_rules(self):
        """Clear all rules from the Polar KB"""
        result = lib.polar_clear_rules(self.ptr)
        process_messages(self.next_message)
        check_result(result)

    def new_query_from_str(self, query_str):
        new_q_ptr = lib.polar_new_query(self.ptr, to_c_str(query_str), 0)
        process_messages(self.next_message)
        query = check_result(new_q_ptr)
        return Query(query)

    def new_query_from_term(self, query_term):
        new_q_ptr = lib.polar_new_query_from_term(
            self.ptr, ffi_serialize(query_term), 0
        )
        process_messages(self.next_message)
        query = check_result(new_q_ptr)
        return Query(query)

    def next_inline_query(self):
        q = lib.polar_next_inline_query(self.ptr, 0)
        process_messages(self.next_message)
        if is_null(q):
            return None
        return Query(q)

    def register_constant(self, value, name):
        name = to_c_str(name)
        value = ffi_serialize(value)
        result = lib.polar_register_constant(self.ptr, name, value)
        process_messages(self.next_message)
        check_result(result)

    def next_message(self):
        return lib.polar_next_polar_message(self.ptr)


class Query:
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
        check_result(lib.polar_call_result(self.ptr, call_id, value))

    def question_result(self, call_id, answer):
        answer = 1 if answer else 0
        check_result(lib.polar_question_result(self.ptr, call_id, answer))

    def application_error(self, message):
        """Pass an error back to polar to get stack trace and other info."""
        message = to_c_str(message)
        check_result(lib.polar_application_error(self.ptr, message))

    def next_event(self):
        event = lib.polar_next_query_event(self.ptr)
        process_messages(self.next_message)
        event = check_result(event)
        return QueryEvent(event)

    def debug_command(self, command):
        result = lib.polar_debug_command(self.ptr, ffi_serialize(command))
        process_messages(self.next_message)
        check_result(result)

    def next_message(self):
        return lib.polar_next_query_message(self.ptr)

    def source(self):
        source = lib.polar_query_source_info(self.ptr)
        source = check_result(source)
        return Source(source)


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

    def get(self):
        return get_python_error(ffi.string(self.ptr).decode())

    def __del__(self):
        lib.string_free(self.ptr)


class Source:
    def __init__(self, ptr):
        self.ptr = ptr

    def get(self):
        return ffi.string(self.ptr).decode()

    def __del__(self):
        lib.string_free(self.ptr)


def check_result(result):
    if result == 0 or is_null(result):
        raise Error().get()
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
            print(msg)
        elif kind == "Warning":
            print(f"[warning] {msg}")
