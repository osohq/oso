import json

from _polar_lib import ffi, lib

from .exceptions import (
    ParserException,
    PolarApiException,
    PolarException,
    PolarOperationalException,
    PolarRuntimeException,
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
            return ParserException(message, data)
        elif kind == "Runtime":
            return PolarRuntimeException(message)
        elif kind == "Operational":
            return PolarOperationalException(message)
        elif kind == "Parameter":
            return PolarApiException(message)
    finally:
        lib.string_free(err_s)
