from contextlib import contextmanager
from dataclasses import dataclass
import json

from _polar_lib import ffi, lib

from .errors import get_python_error
from .exceptions import PolarRuntimeException


class Polar:
    def __init__(self):
        self.ptr = lib.polar_new()

    def __del__(self):
        lib.polar_free(self.ptr)

    def new_id(self):
        """Request a unique ID from the canonical external ID tracker."""
        return check_result(lib.polar_get_external_id(self.ptr))

    def load_str(self, string, filename):
        """Load a Polar string, checking that all inline queries succeed."""
        string = to_c_str(string)
        filename = to_c_str(str(filename)) if filename else ffi.NULL
        check_result(lib.polar_load(self.ptr, string, filename))

    def new_query_from_str(self, query_str):
        return Query(
            check_result(lib.polar_new_query(self.ptr, to_c_str(query_str), 0))
        )

    def new_query_from_term(self, query_term):
        return Query(
            check_result(
                lib.polar_new_query_from_term(self.ptr, ffi_serialize(query_term), 0)
            )
        )

    def next_inline_query(self):
        q = lib.polar_next_inline_query(self.ptr, 0)
        if is_null(q):
            return None
        else:
            return Query(q)

    def register_constant(self, name, value):
        name = to_c_str(name)
        value = ffi_serialize(value)
        check_result(lib.polar_register_constant(self.ptr, name, value))


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
        return QueryEvent(check_result(lib.polar_next_query_event(self.ptr)))

    def debug_command(self, command):
        check_result(lib.polar_debug_command(self.ptr, ffi_serialize(command)))


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
