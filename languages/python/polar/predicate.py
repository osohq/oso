from dataclasses import dataclass
from typing import Any, Sequence


@dataclass(frozen=True)
class Predicate:
    """Represent a predicate in Polar (`name(args, ...)`)."""

    name: str
    args: Sequence[Any]

    def __str__(self):
        return f'{self.name}({", ".join(self.args)})'

    def __eq__(self, other):
        if not isinstance(other, Predicate):
            return False
        return (
            self.name == other.name
            and len(self.args) == len(other.args)
            and all(x == y for x, y in zip(self.args, other.args))
        )
