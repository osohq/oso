from typing import Any, Sequence


class Predicate:
    """Represent a predicate in Polar (`name(args, ...)`)."""

    name: str
    args: Sequence[Any]

    def __init__(self, name: str, args: Sequence[Any]) -> None:
        self.name = name
        self.args = args

    def __str__(self) -> str:
        return f'{self.name}({", ".join(self.args)})'

    def __eq__(self, other: Any) -> bool:
        if not isinstance(other, Predicate):
            return False
        return (
            self.name == other.name
            and len(self.args) == len(other.args)
            and all(x == y for x, y in zip(self.args, other.args))
        )
