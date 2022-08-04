from typing import Any


class Variable(str):
    """An unbound variable type, can be used to query the KB for information"""

    def __repr__(self) -> str:
        return f"Variable({super().__repr__()})"

    def __str__(self) -> str:
        return repr(self)

    def __eq__(self, other: Any) -> bool:
        return super().__eq__(other)

    def __hash__(self) -> int:
        return super().__hash__()
