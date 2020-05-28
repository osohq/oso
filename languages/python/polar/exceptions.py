"""Exceptions used within polar.

assert statements should be avoided unless the violation indicates a
programming error on our part.
"""
# @TODO: Should we just generate these from the rust code?


class PolarException(Exception):
    """Base class for all exceptions from within polar."""

    pass


class ParserException(PolarException):
    """Parse time errors."""

    pass


class IntegerOverflow(ParserException):
    def __init__(self, token, pos):
        self.token = token
        self.pos = pos


class InvalidTokenCharacter(ParserException):
    def __init__(self, token, c, pos):
        self.token = token
        self.c = c
        self.pos = pos


class InvalidToken(ParserException):
    def __init__(self, pos):
        self.pos = pos


class UnrecognizedEOF(ParserException):
    def __init__(self, pos):
        self.pos = pos


class UnrecognizedToken(ParserException):
    def __init__(self, token, pos):
        self.token = token
        self.pos = pos


class ExtraToken(ParserException):
    def __init__(self, token, pos):
        self.token = token
        self.pos = pos


class PolarRuntimeException(PolarException):
    """Exception occuring at runtime (during query tell or evaluation)."""

    pass


class Serialization(PolarRuntimeException):
    def __str__(self):
        return f"Something goes here."


class UnboundVariable(PolarRuntimeException):
    def __str__(self):
        return f"Variable {super().__str__()} is unbound."


class PolarOperationalException(PolarException):
    """Exceptions from polar that are not necessesarily the user's fault. OOM etc..."""

    pass


class Unknown(PolarOperationalException):
    def __init__(self, message):
        self.message = message

    def __str__(self):
        return self.message


class PolarApiException(PolarException):
    """ Exceptions coming from the python bindings to polar, not the engine itself. """

    pass
