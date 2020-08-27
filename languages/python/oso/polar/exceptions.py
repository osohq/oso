"""Exceptions used within polar.
"""
# @TODO: Should we just generate these from the rust code?


class OsoException(Exception):
    def __init__(self, message=None, details=None):
        self.details = details
        self.stack_trace = details.stack_trace
        super(message)


class FFIErrorNotFound(OsoException):
    pass


class PolarRuntimeException(OsoException):
    pass


class SerializationError(PolarRuntimeException):
    pass


class UnsupportedError(PolarRuntimeException):
    pass


class PolarTypeError(PolarRuntimeException):
    pass


class StackOverflowError(PolarRuntimeException):
    pass


class UnregisteredClassError(PolarRuntimeException):
    pass


class MissingConstructorError(PolarRuntimeException):
    pass


class UnregisteredInstanceError(PolarRuntimeException):
    pass


class PolarException(Exception):
    """Base class for all exceptions from within polar."""

    def __init__(self, message, error=None):
        super(PolarException, self).__init__(message)
        self._inner = error


class ParserException(PolarException):
    """Parse time errors."""

    pass


class PolarRuntimeException(PolarException):
    """Exception occurring at runtime (during query tell or evaluation)."""

    def __init__(self, message, error=None):
        super().__init__(message, error)
        self.stack_trace = None
        if error:
            self.kind = [*error][0]
            data = error[self.kind]
            if "stack_trace" in data:
                self.stack_trace = data["stack_trace"]

    def __str__(self):
        return super(PolarException, self).__str__()


class InlineQueryFailedError(PolarRuntimeException):
    pass


class PolarOperationalException(PolarException):
    """Exceptions from polar that are not necessarily the user's fault. OOM etc..."""

    pass


class PolarApiException(PolarException):
    """ Exceptions coming from the python bindings to polar, not the engine itself. """

    pass


class IntegerOverflow(ParserException):
    pass


class InvalidTokenCharacter(ParserException):
    pass


class InvalidToken(ParserException):
    pass


class UnrecognizedEOF(ParserException):
    pass


class UnrecognizedToken(ParserException):
    pass


class ExtraToken(ParserException):
    pass
