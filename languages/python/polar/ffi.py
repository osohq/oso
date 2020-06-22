from contextlib import contextmanager
from dataclasses import dataclass
import json

from _polar_lib import ffi, lib
from typing import Any, Sequence

from .errors import raise_error
from .exceptions import PolarRuntimeException


class Variable(str):
    """An unbound variable type, can be used to query the KB for information"""

    pass


@dataclass(frozen=True)
class Predicate:
    """Represent a predicate in Polar (`name(args, ...)`)."""

    name: str
    args: Sequence[Any]

    def __str__(self):
        return f'{self.name}({", ".join(self.args)})'

    def __eq__(self, other):
        if not isinstance(other, Predicate):
            return False
        return (
            self.name == other.name
            and len(self.args) == len(other.args)
            and all(x == y for x, y in zip(self.args, other.args))
        )


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


@contextmanager
def manage_query(query):
    """Context manager for Polar queries."""
    try:
        yield query
    finally:
        lib.query_free(query)


def load_str(polar, string, filename, do_query):
    """Load a Polar string, checking that all inline queries succeed."""
    string = to_c_str(string)
    filename = to_c_str(str(filename)) if filename else ffi.NULL
    check_result(lib.polar_load(polar, string, filename))

    # check inline queries
    while True:
        query = lib.polar_next_inline_query(polar)
        if is_null(query):  # Load is done
            break
        else:
            try:
                next(do_query(query))
            except StopIteration:
                raise PolarRuntimeException("Inline query in file failed.")


def new_id(polar):
    """Request a unique ID from the canonical external ID tracker."""
    return check_result(lib.polar_get_external_id(polar))


def external_call(polar, query, call_id, value):
    """Make an external call and propagate FFI errors."""
    if value is None:
        value = ffi.NULL
    check_result(lib.polar_call_result(query, call_id, value))


def external_answer(polar, query, call_id, answer):
    answer = 1 if answer else 0
    check_result(lib.polar_question_result(query, call_id, answer))
