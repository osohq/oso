"""Exceptions used within polar.

assert statements should be avoided unless the violation indicates a
programming error on our part.
"""
# @TODO: Should we just generate these from the rust code?


class PolarException(Exception):
    """Base class for all exceptions from within polar."""

    def __init__(self, message, error=None):
        super(PolarException, self).__init__(message)
        self._inner = error


class ParserException(PolarException):
    """Parse time errors."""

    pass


class PolarRuntimeException(PolarException):
    """Exception occuring at runtime (during query tell or evaluation)."""

    pass


class PolarOperationalException(PolarException):
    """Exceptions from polar that are not necessesarily the user's fault. OOM etc..."""

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
