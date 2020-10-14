import json

from _polar_lib import ffi, lib
from .exceptions import *


def get_python_error(err_str):
    """Fetch a Polar error and map it into a Python exception."""
    err = json.loads(err_str)

    message = err["formatted"]
    kind, body = next(iter(err["kind"].items()))

    try:
        subkind, details = next(iter(body.items()))
    except (AttributeError, TypeError, StopIteration):
        # Not all errors have subkind and details.
        # TODO (dhatch): This bug may exist in other libraries.
        subkind = None
        details = None

    if kind == "Parse":
        return _parse_error(subkind, message, details)
    elif kind == "Runtime":
        return _runtime_error(subkind, message, details)
    elif kind == "Operational":
        return _operational_error(subkind, message, details)
    elif kind == "Parameter":
        return _api_error(message, details)


def _parse_error(subkind, message, details):
    """Map parsing errors."""
    parse_errors = {
        "ExtraToken": ExtraToken(message, details),
        "IntegerOverflow": IntegerOverflow(message, details),
        "InvalidToken": InvalidToken(message, details),
        "InvalidTokenCharacter": InvalidTokenCharacter(message, details),
        "UnrecognizedEOF": UnrecognizedEOF(message, details),
        "UnrecognizedToken": UnrecognizedToken(message, details),
    }
    return parse_errors.get(subkind, ParserError(message, details))


def _runtime_error(subkind, message, details):
    runtime_errors = {
        "Serialization": SerializationError(message, details),
        "Unsupported": UnsupportedError(message, details),
        "TypeError": PolarTypeError(message, details),
        "StackOverflow": StackOverflowError(message, details),
        "FileLoading": FileLoadingError(message, details),
    }
    return runtime_errors.get(subkind, PolarRuntimeError(message, details))


def _operational_error(subkind, message, details):
    if subkind == "Unknown":
        return UnknownError(message, details)
    else:
        return OperationalError(message, details)


def _api_error(subkind, message, details):
    if subkind == "Parameter":
        return ParameterError(message, details)
    else:
        return PolarApiError(message, details)
    pass
