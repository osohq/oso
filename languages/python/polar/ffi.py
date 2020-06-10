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
        return f'{self.name}({self.args.join(", ")})'

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


def to_polar_term(v, new_id):
    """Convert Python values to Polar terms."""
    if isinstance(v, bool):
        val = {"Boolean": v}
    elif isinstance(v, int):
        val = {"Integer": v}
    elif isinstance(v, str):
        val = {"String": v}
    elif isinstance(v, list):
        val = {"List": [to_polar_term(i, new_id) for i in v]}
    elif isinstance(v, dict):
        val = {
            "Dictionary": {
                "fields": {k: to_polar_term(v, new_id) for k, v in v.items()}
            }
        }
    elif isinstance(v, Predicate):
        val = {
            "Call": {"name": v.name, "args": [to_polar_term(v, new_id) for v in v.args]}
        }
    elif isinstance(v, Variable):
        # This is supported so that we can query for unbound variables
        val = {"Symbol": v}
    else:
        val = {"ExternalInstance": {"instance_id": new_id(v)}}
    term = {"id": 0, "offset": 0, "value": val}
    return term


def stringify(value, new_id):
    formatted = to_polar_term(value, new_id)
    dumped = json.dumps(formatted)
    return to_c_str(dumped)


@contextmanager
def polar_query(query):
    """Context manager for Polar queries."""
    try:
        yield query
    finally:
        lib.query_free(query)


def unstringify(string):
    """Reconstruct Python object from JSON-encoded C string."""
    try:
        check_result(string)
        return json.loads(ffi.string(string).decode())
    finally:
        if not is_null(string):
            lib.string_free(string)


def load_str(polar, string, do_query):
    """Load a Polar string, checking that all inline queries succeed."""
    string = to_c_str(string)
    load = check_result(lib.polar_new_load(polar, string))
    try:
        _check_inline_queries(polar, load, do_query)
    finally:
        lib.load_free(load)


def _check_inline_queries(polar, load, do_query):
    while True:
        query = ffi.new("polar_Query **")
        check_result(lib.polar_load(polar, load, query))
        if query[0] == ffi.NULL:  # Load is done
            break
        else:
            try:
                next(do_query(query[0]))
            except StopIteration:
                raise PolarRuntimeException("Inline query in file failed.")


def external_call(polar, query, call_id, value):
    """Make an external call and propagate FFI errors."""
    if value is None:
        value = ffi.NULL
    check_result(lib.polar_external_call_result(polar, query, call_id, value))


def new_id(polar):
    """Request a unique ID from the canonical external ID tracker."""
    return check_result(lib.polar_get_external_id(polar))


def external_answer(polar, query, call_id, answer):
    answer = 1 if answer else 0
    check_result(lib.polar_external_question_result(polar, query, call_id, answer))
