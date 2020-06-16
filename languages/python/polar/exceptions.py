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
    def __init__(self, token, context):
        self.token = token
        self.context = context


class InvalidTokenCharacter(ParserException):
    def __init__(self, token, c, context):
        self.token = token
        self.c = c
        self.context = context


class InvalidToken(ParserException):
    def __init__(self, context):
        self.context = context


class UnrecognizedEOF(ParserException):
    def __init__(self, context):
        self.context = context


class UnrecognizedToken(ParserException):
    def __init__(self, token, context):
        self.token = token
        self.context = context


class ExtraToken(ParserException):
    def __init__(self, token, context):
        self.token = token
        self.context = context


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
