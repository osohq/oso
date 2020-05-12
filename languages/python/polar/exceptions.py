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


class PolarRuntimeException(PolarException):
    """Exception occuring at runtime (during query tell or evaluation)."""

    pass


class UnboundVariable(PolarRuntimeException):
    def __str__(self):
        return f"Variable {super().__str__()} is unbound."
