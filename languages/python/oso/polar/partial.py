from .expression import Expression, Pattern
from .variable import Variable


class TypeConstraint(Expression):
    def __init__(self, left, type_name):
        self.operator = "Isa"
        self.args = [left, Pattern(type_name, {})]


def dot_path(expr):
    """Get the path components of a (potentially nested) dot lookup. The path
    is returned as a tuple. The empty tuple is returned if input is not a dot
    operation.

    _this => ()
    _this.created_by => ('created_by',)
    _this.created_by.username => ('created_by', 'username')"""

    if isinstance(expr, Variable) and expr != Variable("_this"):
        return (expr,)

    if not (isinstance(expr, Expression) and expr.operator == "Dot"):
        return ()

    (left, right) = expr.args

    if left == Variable("_this"):
        return (right,)

    return dot_path(left) + (right,)
