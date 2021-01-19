from typing import Tuple

from .expression import Expression, Pattern
from .variable import Variable


# And(Isa(left, right))
class TypeConstraint(Expression):
    def __init__(self, left, type_name):
        self.operator = "And"
        self.args = [Expression("Isa", [left, Pattern(type_name, {})])]


# pylint: disable=E1136 # PyCQA/pylint/issues/3882
def dot_path(
    expr,
) -> Tuple[Variable, ...]:
    """Get the path components of a (potentially nested) dot lookup. The path
    is returned as a tuple. The empty tuple is returned if input is not a dot
    operation.

    _this => (_this,)
    _this.created_by => (_this, 'created_by',)
    _this.created_by.username => (_this, 'created_by', 'username')"""

    if isinstance(expr, Variable):
        return (expr,)
    elif not (isinstance(expr, Expression) and expr.operator == "Dot"):
        return ()
    (left, right) = expr.args
    return dot_path(left) + (right,)
