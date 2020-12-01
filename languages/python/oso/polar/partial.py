from .expression import Expression
from .variable import Variable


class Partial:
    """A partial variable."""

    def __init__(self, name, *constraints):
        self.name = name
        self.constraints = constraints

    def __repr__(self):
        return f"Partial({self.name})"

    def __str__(self):
        return repr(self)

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.name == other.name
            and self.constraints == other.constraints
        )

    def to_polar(self):
        return {
            "variable": self.name,
            "constraints": [c.to_polar() for c in self.constraints],
        }


class Constraint:
    pass


class TypeConstraint(Constraint):
    def __init__(self, type_name):
        self.type_name = type_name

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.type_name == other.type_name

    def to_polar(self):
        return {
            "operator": "Isa",
            "args": [
                {"value": {"Variable": "_this"}},
                {
                    "value": {
                        "Pattern": {
                            "Instance": {
                                "tag": self.type_name,
                                "fields": {"fields": {}},
                            }
                        }
                    }
                },
            ],
        }


def dot_path(expr):
    """Get the path components of a (potentially nested) dot lookup. The path
    is returned as a tuple. The empty tuple is returned if input is not a dot
    operation.

    _this => ()
    _this.created_by => ('created_by',)
    _this.created_by.username => ('created_by', 'username')"""

    if not (isinstance(expr, Expression) and expr.operator == "Dot"):
        return ()

    (left, right) = expr.args

    if left == Variable("_this"):
        return (right,)

    return dot_path(left) + (right,)
