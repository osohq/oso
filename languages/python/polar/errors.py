import json

from _polar_lib import ffi, lib

from .exceptions import (
    ExtraToken,
    IntegerOverflow,
    InvalidToken,
    InvalidTokenCharacter,
    ParserException,
    PolarRuntimeException,
    Unknown,
    UnrecognizedEOF,
    UnrecognizedToken,
)


def raise_error():
    raise get_error()


def get_error():
    """Fetch a Polar error and map it into a Python exception."""
    try:
        err_s = lib.polar_get_error()
        err_json = ffi.string(err_s).decode()
        err = json.loads(err_json)
        kind = [*err][0]
        data = err[kind]

        if kind == "Parse":
            subkind = [*data][0]
            return _parse_error(subkind, data)
        elif kind == "Runtime":  # @TODO: Runtime exception types.
            return PolarRuntimeException(json.dumps(data))
        elif kind == "Operational":
            subkind = [*data][0]
            if subkind == "Unknown":  # Rust panic.
                return Unknown("Unknown Internal Error: See console.")
        # All errors should be mapped to python exceptions.
        # Raise Unknown if we haven't mapped the error.
        return Unknown(f"Unknown Internal Error: {err_json}")
    finally:
        lib.string_free(err_s)


def _parse_error(kind, data):
    """Map parsing errors."""
    token = data[kind].get("token")
    context = data[kind].get("context")
    c = data[kind].get("c")
    parse_errors = {
        "ExtraToken": ExtraToken(token, context),
        "IntegerOverflow": IntegerOverflow(token, context),
        "InvalidToken": InvalidToken(context),
        "InvalidTokenCharacter": InvalidTokenCharacter(token, c, context),
        "UnrecognizedEOF": UnrecognizedEOF(context),
        "UnrecognizedToken": UnrecognizedToken(token, context),
    }
    return parse_errors.get(
        kind, ParserException(f"Parser Exception: {json.dumps(data)}")
    )
