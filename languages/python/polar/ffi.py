from contextlib import contextmanager
from dataclasses import dataclass
import json

from _polar_lib import ffi, lib

from .errors import raise_error
from .exceptions import PolarRuntimeException


def check_result(result):
    if result == 0 or is_null(result):
        raise_error()
    return result


def is_null(result):
    return result == ffi.NULL


def to_c_str(string):
    return ffi.new("char[]", string.encode())


def ffi_serialize(value):
    return to_c_str(json.dumps(value))


def ffi_deserialize(string):
    """Reconstruct Python object from JSON-encoded C string."""
    try:
        check_result(string)
        return json.loads(ffi.string(string).decode())
    finally:
        if not is_null(string):
            lib.string_free(string)


def load_str(polar, string, filename):
    """Load a Polar string, checking that all inline queries succeed."""
    string = to_c_str(string)
    filename = to_c_str(str(filename)) if filename else ffi.NULL
    check_result(lib.polar_load(polar, string, filename))


def new_id(polar):
    """Request a unique ID from the canonical external ID tracker."""
    return check_result(lib.polar_get_external_id(polar))


def external_call(query, call_id, value):
    """Make an external call and propagate FFI errors."""
    if value is None:
        value = ffi.NULL
    check_result(lib.polar_call_result(query, call_id, value))


def external_answer(query, call_id, answer):
    answer = 1 if answer else 0
    check_result(lib.polar_question_result(query, call_id, answer))


def application_error(query, message):
    """Pass an error back to polar to get stack trace and other info."""
    message = to_c_str(message)
    check_result(lib.polar_application_error(query, message))
