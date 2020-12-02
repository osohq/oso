import functools
from typing import Any, Callable, Tuple

from sqlalchemy.orm.session import Session
from sqlalchemy.orm.query import Query
from sqlalchemy import inspect
from sqlalchemy.orm import RelationshipProperty, ColumnProperty
from sqlalchemy.sql.expression import ClauseElement, BinaryExpression, and_
from sqlalchemy.sql import expression as sql

from polar.partial import dot_path
from polar.expression import Expression
from polar.variable import Variable
from polar.exceptions import UnsupportedError

# TODO (dhatch) Better types here, first any is model, second any is a sqlalchemy expr.
EmitFunction = Callable[[Session, Any], Any]


def partial_to_filter(expression: Expression, session: Session, model, get_model):
    """Convert constraints in ``partial`` to a filter over ``model`` that should be applied to query."""
    return translate_expr(expression, session, model, get_model)


# Returns None or the translated expression.
def translate_expr(expression: Expression, session: Session, model, get_model):
    assert isinstance(expression, Expression)
    if expression.operator == "Eq" or expression.operator == "Unify":
        return translate_compare(expression, session, model, get_model)
    elif expression.operator == "Isa":
        return translate_isa(expression, session, model, get_model)
    elif expression.operator == "In":
        return translate_in(expression, session, model, get_model)
    elif expression.operator == "And":
        return translate_and(expression, session, model, get_model)
    else:
        raise UnsupportedError(f"Unsupported {expression}")


def translate_and(expression: Expression, session: Session, model, get_model):
    assert expression.operator == "And"
    expr = and_()
    for expression in expression.args:
        translated = translate_expr(expression, session, model, get_model)
        if translated is None:
            continue

        expr = expr & translated

    return expr


def translate_isa(expression: Expression, session: Session, model, get_model):
    assert expression.operator == "Isa"
    left, right = expression.args
    if dot_path(left) == ():
        assert left == Variable("_this")
    else:
        for field_name in dot_path(left):
            _, model, __ = get_relationship(model, field_name)

    assert not right.fields, "Unexpected fields in isa expression"
    constraint_type = get_model(right.tag)
    if not issubclass(model, constraint_type):
        return sql.false()
    else:
        return None


def translate_compare(expression: Expression, session: Session, model, get_model):
    left = expression.args[0]
    right = expression.args[1]

    left_path = dot_path(left)
    if left_path:
        path = left_path
        value = right
    else:
        assert left == Variable("_this")
        assert inspect(right)

        primary_keys = [pk.name for pk in inspect(model).primary_key]
        pk_filter = None
        for key in primary_keys:
            key_value = getattr(right, key)
            if pk_filter is None:
                pk_filter = getattr(model, key) == key_value
            else:
                pk_filter &= getattr(model, key) == key_value

        return pk_filter

    path, field_name = path[:-1], path[-1]
    return translate_dot(
        path, session, model, functools.partial(emit_compare, field_name, value)
    )


def translate_in(expression, session, model, get_model):
    assert expression.operator == "In"
    left = expression.args[0]
    right = expression.args[1]

    # IN means at least something must be contained in the property.

    # There are two possible types of in operations. In both, the right hand side
    # should be a dot op.

    # Partial In: LHS is an expression
    if isinstance(left, Expression):
        path = dot_path(right)
        assert path

        return translate_dot(
            path, session, model, functools.partial(emit_subexpression, left, get_model)
        )
    else:
        # Contains: LHS is not an expression.
        # TODO (dhatch) Missing check, left type must match type of the target?
        path = dot_path(right)
        assert path
        path, field_name = path[:-1], path[-1]
        return translate_dot(
            path, session, model, functools.partial(emit_contains, field_name, left)
        )


def translate_dot(path: Tuple[str], session: Session, model, func: EmitFunction):
    if len(path) == 0:
        return func(session, model)
    else:
        property, model, is_multi_valued = get_relationship(model, path[0])
        if not is_multi_valued:
            return property.has(translate_dot(path[1:], session, model, func))
        else:
            return property.any(translate_dot(path[1:], session, model, func))


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


def emit_subexpression(sub_expression: Expression, get_model, session: Session, model):
    """Emit a sub-expression on ``model``."""
    return translate_expr(sub_expression, session, model, get_model)


def emit_contains(field_name, value, session, model):
    """Emit a contains operation, checking that multi-valued relationship field ``field_name`` contains ``value``."""
    # TODO (dhatch): Could this be valid for fields that are not relationship fields?
    property, model, is_multi_valued = get_relationship(model, field_name)
    assert is_multi_valued

    return property.contains(value)
