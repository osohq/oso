import functools
from typing import Any, Callable, List

from sqlalchemy.orm.session import Session
from sqlalchemy.orm.query import Query
from sqlalchemy.orm import RelationshipProperty, ColumnProperty
from sqlalchemy.sql.expression import ClauseElement, BinaryExpression, and_

from polar.partial import Partial
from polar.expression import Expression
from polar.variable import Variable
from polar.exceptions import UnsupportedError

# TODO (dhatch) Better types here, first any is model, second any is a sqlalchemy expr.
EmitFunction = Callable[[Session, Any], Any]

def partial_to_query(expression: Expression, session: Session, model) -> Query:
    """Convert constraints in ``partial`` to a query over ``model``."""
    # Top level operation must be and.
    query = session.query(model)

    print(expression)

    expr = translate_expr(expression, session, model)
    if expr is not None:
        return query.filter(expr)

    return query

# Returns None or the translated expression.
def translate_expr(expression: Expression, session: Session, model):
    assert isinstance(expression, Expression)
    if expression.operator == 'Eq' or expression.operator == 'Unify':
        return translate_compare(expression, session, model)
    elif expression.operator == 'Isa':
        assert expression.args[1].tag == model.__name__
        return None
    elif expression.operator == 'In':
        return translate_in(expression, session, model)
    elif expression.operator == 'And':
        return translate_and_expr(expression, session, model)
    else:
        raise UnsupportedError(f"Unsupported {expression}")

def translate_and_expr(expression: Expression, session: Session, model):
    expr = and_()
    assert expression.operator == "And"
    for expression in expression.args:
        translated = translate_expr(expression, session, model)
        if translated is None:
            continue

        expr = expr & translated

    return expr

def translate_compare(expression: Expression, session: Session, model):
    left = expression.args[0]
    right = expression.args[1]

    if dot_op_path(left):
        path = dot_op_path(left)
        value = right
    else:
        path = dot_op_path(right)
        assert path
        value = left

    path, field_name = path[:-1], path[-1]
    return translate_dot_op(
        path,
        session,
        model,
        functools.partial(emit_compare, field_name, value))

def translate_in(expression, session, model):
    assert expression.operator == 'In'
    left = expression.args[0]
    right = expression.args[1]

    # IN means at least something must be contained in the property.

    # There are two possible types of in operations. In both, the right hand side
    # should be a dot op.

    # Partial In: LHS is an expression
    if isinstance(left, Expression):
        path = dot_op_path(right)
        assert path

        return translate_dot_op(
            path,
            session,
            model,
            functools.partial(emit_subexpression, left))
    else:
        # Contains: LHS is not an expression.
        # TODO (dhatch) Missing check, left type must match type of the target?
        path = dot_op_path(right)
        assert path
        path, field_name = path[:-1], path[-1]
        return translate_dot_op(
            path,
            session,
            model,
            functools.partial(emit_contains, field_name, left))

def translate_dot_op(path: List[str], session: Session, model, func: EmitFunction):
    if len(path) == 0:
        return func(session, model)
    else:
        property, model, is_multi_valued = get_relationship(model, path[0])
        if not is_multi_valued:
            return property.has(translate_dot_op(path[1:], session, model, func))
        else:
            return property.any(translate_dot_op(path[1:], session, model, func))

def get_relationship(model, field_name: str):
    """Get the property object for field on model. field must be a relationship field.

    :returns: (property, model, is_multi_valued)
    """
    property = getattr(model, field_name)
    assert isinstance(property.property, RelationshipProperty)
    relationship = property.property
    model = property.entity.class_

    return (property, model, relationship.uselist)

def emit_compare(field_name, value, session, model):
    """Emit a comparison operation comparing the value of ``field_name`` on ``model`` to ``value``."""
    property = getattr(model, field_name)
    return property == value

def emit_subexpression(sub_expression: Expression, session: Session, model):
    """Emit a sub-expression on ``model``."""
    return translate_expr(sub_expression, session, model)

def emit_contains(field_name, value, session, model):
    """Emit a contains operation, checking that multi-valued relationship field ``field_name`` contains ``value``."""
    # TODO (dhatch): Could this be valid for fields that are not relationship fields?
    property, model, is_multi_valued = get_relationship(model, field_name)
    assert is_multi_valued

    return property.contains(value)

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
