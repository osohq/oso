import json

from polar.exceptions import (
    ExtraToken,
    IntegerOverflow,
    InvalidToken,
    UnrecognizedToken,
    PolarTypeError,
    StackOverflowError,
    PolarRuntimeError,
    UnknownError,
    OperationalError,
    InvalidTokenCharacter,
    UnrecognizedEOF,
    ParserError,
    UnsupportedError,
    ValidationError,
)


def get_python_error(err_str, enrich_message=None):
    """Fetch a Polar error and map it into a Python exception."""
    err = json.loads(err_str)

    message = err["formatted"]
    if enrich_message:
        message = enrich_message(message)
    kind, body = next(iter(err["kind"].items()))

    try:
        subkind, details = next(iter(body.items()))
    except (AttributeError, TypeError, StopIteration):
        # Not all errors have subkind and details.
        # TODO (dhatch): This bug may exist in other libraries.
        subkind = None
        details = None

    if details and enrich_message:
        if details.get("stack_trace"):
            details["stack_trace"] = enrich_message(details["stack_trace"])
        if "msg" in details:
            details["msg"] = enrich_message(details["msg"])

    if kind == "Parse":
        return _parse_error(subkind, message, details)
    elif kind == "Runtime":
        return _runtime_error(subkind, message, details)
    elif kind == "Operational":
        return _operational_error(subkind, message, details)
    elif kind == "Validation":
        return _validation_error(message, details)


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
        "Unsupported": UnsupportedError(message, details),
        "TypeError": PolarTypeError(message, details),
        "StackOverflow": StackOverflowError(message, details),
    }
    return runtime_errors.get(subkind, PolarRuntimeError(message, details))


def _operational_error(subkind, message, details):
    if subkind == "Unknown":
        return UnknownError(message, details)
    else:
        return OperationalError(message, details)


def _validation_error(message, details):
    return ValidationError(message, details)
