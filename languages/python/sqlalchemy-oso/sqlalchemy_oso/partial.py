

from sqlalchemy.orm.session import Session
from sqlalchemy.orm.query import Query
from sqlalchemy.sql.expression import ClauseElement, BinaryExpression, and_

from polar.partial import Partial
from polar.expression import Expression
from polar.variable import Variable
from polar.exceptions import UnsupportedError

def partial_to_query(expression: Expression, session: Session, model) -> Query:
    """Convert constraints in ``partial`` to a query over ``model``."""
    # Top level operation must be and.
    query = session.query(model)

    print(expression)

    expr = and_expr(expression, session, model)
    return query.filter(expr)

def and_expr(expression: Expression, session: Session, model) -> BinaryExpression:
    expr = and_()
    assert expression.operator == "And"
    for expression in expression.args:
        assert isinstance(expression, Expression)
        if expression.operator == 'Eq' or expression.operator == 'Unify':
            expr = expr & compare_expr(expression, session, model)
        elif expression.operator == 'Isa':
            assert expression.args[1].tag == model.__name__
        elif expression.operator == 'And':
            expr = expr & and_expr(expression, session, model)
        else:
            raise UnsupportedError(f"Unsupported {expression}")

    # TODO (dhatch) Maybe this just returns the where part ? But we may need to
    # add joins.
    return expr

def compare_expr(expression: Expression, session: Session, model) -> BinaryExpression:
    left = expression.args[0]
    right = expression.args[1]

    if dot_op_field(left):
        field = dot_op_field(left)
        value = right
        # TODO non eq
        # TODO is there a better way to do this that isn't getattr
        return getattr(model, field) == value
    else:
        field = dot_op_field(right)
        assert field
        value = left
        return getattr(model, field) == value

# TODO (dhatch): Move this helper into base.
def dot_op_field(expr):
    """Get the field from dot op ``expr`` or return ``False``."""
    return (
        isinstance(expr, Expression)
        and expr.operator == "Dot"
        and isinstance(expr.args[0], Variable)
        and expr.args[0] == Variable("_this")
        and expr.args[1]
    )
