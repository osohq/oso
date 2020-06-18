import json

from _polar_lib import ffi, lib

from .exceptions import (
    ParserException,
    PolarApiException,
    PolarException,
    PolarOperationalException,
    PolarRuntimeException,
    IntegerOverflow,
    InvalidTokenCharacter,
    InvalidToken,
    UnrecognizedEOF,
    UnrecognizedToken,
    ExtraToken,
)


def raise_error():
    raise get_error()


def get_error():
    """Fetch a Polar error and map it into a Python exception."""
    try:
        err_s = lib.polar_get_error()
        err_json = ffi.string(err_s).decode()
        err = json.loads(err_json)

        kind = [*err["kind"]][0]
        data = err["kind"][kind]
        message = err["formatted"]

        if kind == "Parse":
            return _parse_error(message, data)
        elif kind == "Runtime":
            return PolarRuntimeException(message, data)
        elif kind == "Operational":
            return PolarOperationalException(message, data)
        elif kind == "Parameter":
            return PolarApiException(message, data)
    finally:
        lib.string_free(err_s)


def _parse_error(message, data):
    """Map parsing errors."""
    kind = [*data][0]
    data = data[kind]
    parse_errors = {
        "ExtraToken": ExtraToken(message, data),
        "IntegerOverflow": IntegerOverflow(message, data),
        "InvalidToken": InvalidToken(message, data),
        "InvalidTokenCharacter": InvalidTokenCharacter(message, data),
        "UnrecognizedEOF": UnrecognizedEOF(message, data),
        "UnrecognizedToken": UnrecognizedToken(message, data),
    }
    return parse_errors.get(kind, ParserException(message, data))
