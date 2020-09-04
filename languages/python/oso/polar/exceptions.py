"""Exceptions used within polar.
"""
# @TODO: Should we just generate these from the rust code?


class OsoError(Exception):
    def __init__(self, message=None, details=None):
        self.message = message
        self.details = details
        self.stack_trace = details.get("stack_trace") if details else None
        super().__init__(self.message)


class FFIErrorNotFound(OsoError):
    pass


# ==================
# RUNTIME EXCEPTIONS
# ==================


class PolarRuntimeError(OsoError):
    pass


class SerializationError(PolarRuntimeError):
    pass


class UnsupportedError(PolarRuntimeError):
    pass


class PolarTypeError(PolarRuntimeError):
    pass


class StackOverflowError(PolarRuntimeError):
    pass


class FileLoadingError(PolarRuntimeError):
    pass


class UnregisteredClassError(PolarRuntimeError):
    pass


class DuplicateClassAliasError(PolarRuntimeError):
    def __init__(self, name, old, new):
        super().__init__(
            f"Attempted to alias {new} as '{name}', but {old} already has that alias."
        )


class DuplicateInstanceRegistrationError(PolarRuntimeError):
    pass


class MissingConstructorError(PolarRuntimeError):
    pass


class UnregisteredInstanceError(PolarRuntimeError):
    pass


class InlineQueryFailedError(PolarRuntimeError):
    pass


class UnexpectedPolarTypeError(PolarRuntimeError):
    pass


# =================
# PARSER EXCEPTIONS
# =================


class ParserError(OsoError):
    """Parse time errors."""

    pass


class IntegerOverflow(ParserError):
    pass


class InvalidTokenCharacter(ParserError):
    pass


class InvalidToken(ParserError):
    pass


class UnrecognizedEOF(ParserError):
    pass


class UnrecognizedToken(ParserError):
    pass


class ExtraToken(ParserError):
    pass


# ======================
# OPERATIONAL EXCEPTIONS
# ======================


class OperationalError(OsoError):
    """Errors from polar that are not necessarily the user's fault. OOM etc..."""

    pass


class UnknownError(OperationalError):
    pass


# ==============
# API EXCEPTIONS
# ==============


class PolarApiError(OsoError):
    """ Errors coming from the python bindings to polar, not the engine itself. """

    pass


class ParameterError(PolarApiError):
    pass