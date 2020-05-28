"""Exceptions used within polar.

assert statements should be avoided unless the violation indicates a
programming error on our part.
"""


class PolarException(Exception):
    """Base class for all exceptions from within polar."""

    pass


class ParserException(PolarException):
    """Parse time errors."""

    pass


class InvalidTokenCharacter(ParserException):
    def __init__(self, c, loc):
        self.c = c
        self.loc = loc

    def __str__(self):
        return f"Invalid Token character '{self.c}' at location {self.loc}"


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
