

from sqlalchemy.orm.session import Session
from sqlalchemy.orm.query import Query
from sqlalchemy.orm import RelationshipProperty, ColumnProperty
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

    if dot_op_path(left):
        path = dot_op_path(left)
        value = right
    else:
        path = dot_op_path(right)
        assert path
        value = left

    return translate_comparison(path, value, model)

def translate_comparison(path, value, model):
    """Translate a comparison operation of ``path`` = ``value`` on ``model``."""
    if len(path) == 1:
        property = getattr(model, path[0])
        return property == value
    else:
        # TODO this has assumes that nested relationships are always
        # a scalar attribute... it also probably isn't as efficient as a
        # join usually, so we may want to translate differently.
        property = getattr(model, path[0])
        assert isinstance(property.property, RelationshipProperty)
        relationship = property.property

        if not relationship.uselist:
            return property.has(
                translate_comparison(path[1:], value, property.entity.class_))
        else:
            return property.any(
                translate_comparison(path[1:], value, property.entity.class_))


# TODO (dhatch): Move this helper into base.
def dot_op_path(expr):
    """Get the path components of a lookup.

    The path is returned as a list.

    _this.created_by => ['created_by']
    _this.created_by.username => ['created_by', 'username']

    None is returned if input is not a dot operation.
    """
    if not isinstance(expr, Expression):
        return None

    if not expr.operator == "Dot":
        return None

    assert len(expr.args) == 2

    if expr.args[0] == Variable('_this'):
        return [expr.args[1]]

    return dot_op_path(expr.args[0]) + [expr.args[1]]
