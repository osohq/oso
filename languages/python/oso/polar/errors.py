import json

from polar.exceptions import (
    ExtraToken,
    IntegerOverflow,
    InvalidToken,
    UnrecognizedToken,
    SerializationError,
    PolarTypeError,
    StackOverflowError,
    FileLoadingError,
    PolarRuntimeError,
    UnknownError,
    OperationalError,
    InvalidTokenCharacter,
    UnrecognizedEOF,
    ParserError,
    UnsupportedError,
    ParameterError,
    PolarApiError,
    RolesValidationError,
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

    if details:
        if "stack_trace" in details:
            details["stack_trace"] = enrich_message(details["stack_trace"])
        if "msg" in details:
            details["msg"] = enrich_message(details["msg"])

    if kind == "Parse":
        return _parse_error(subkind, message, details)
    elif kind == "Runtime":
        return _runtime_error(subkind, message, details)
    elif kind == "Operational":
        return _operational_error(subkind, message, details)
    elif kind == "Parameter":
        # TODO(gj): this is wrong -- method has arity 3.
        return _api_error(message, details)
    elif kind == "RolesValidation":
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


def _validation_error(message, details):
    return RolesValidationError(message, details)


def _api_error(subkind, message, details):
    if subkind == "Parameter":
        return ParameterError(message, details)
    else:
        return PolarApiError(message, details)
