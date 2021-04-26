import functools
from typing import Any, Callable, Tuple

from sqlalchemy.orm.session import Session
from sqlalchemy import inspect
from sqlalchemy.orm import RelationshipProperty
from sqlalchemy.sql import expression as sql
from sqlalchemy.sql.elements import True_

from polar.partial import dot_path
from polar.expression import Expression
from polar.variable import Variable
from polar.exceptions import UnsupportedError

from sqlalchemy_oso.preprocess import preprocess

# TODO (dhatch) Better types here, first any is model, second any is a sqlalchemy expr.
EmitFunction = Callable[[Session, Any], Any]


COMPARISONS = {
    "Unify": lambda p, v: p == v,
    "Eq": lambda p, v: p == v,
    "Neq": lambda p, v: p != v,
    "Geq": lambda p, v: p >= v,
    "Gt": lambda p, v: p > v,
    "Leq": lambda p, v: p <= v,
    "Lt": lambda p, v: p < v,
}


def flip_op(operator):
    flips = {
        "Eq": "Eq",
        "Unify": "Unify",
        "Neq": "Neq",
        "Geq": "Leq",
        "Gt": "Lt",
        "Leq": "Gtq",
        "Lt": "Gt",
    }
    return flips[operator]


def and_filter(current, new):
    if isinstance(current, True_):
        return new
    else:
        return current & new


def partial_to_filter(expression: Expression, session: Session, model, get_model):
    """Convert constraints in ``partial`` to a filter over ``model`` that should be applied to query."""
    expression = preprocess(expression)
    return translate_expr(expression, session, model, get_model)


def translate_expr(expression: Expression, session: Session, model, get_model):
    assert isinstance(expression, Expression)
    if expression.operator in COMPARISONS:
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
    expr = sql.true()
    for expression in expression.args:
        translated = translate_expr(expression, session, model, get_model)
        expr = and_filter(expr, translated)

    return expr


def translate_isa(expression: Expression, session: Session, model, get_model):
    assert expression.operator == "Isa"
    left, right = expression.args
    left_path = dot_path(left)
    assert left_path[0] == Variable("_this")
    left_path = left_path[1:]  # Drop _this.
    if left_path:
        for field_name in left_path:
            _, model, __ = get_relationship(model, field_name)

    assert not right.fields, "Unexpected fields in isa expression"
    constraint_type = get_model(right.tag)
    model_type = inspect(model, raiseerr=True).class_
    return sql.true() if issubclass(model_type, constraint_type) else sql.false()


def translate_compare(expression: Expression, session: Session, model, get_model):
    (left, right) = expression.args
    left_path = dot_path(left)
    right_path = dot_path(right)

    if left_path[1:]:
        assert left_path[0] == Variable("_this")
        assert not right_path
        path, field_name = left_path[1:-1], left_path[-1]
        return translate_dot(
            path,
            session,
            model,
            functools.partial(emit_compare, field_name, right, expression.operator),
        )
    elif right_path and right_path[0] == "_this":
        return translate_compare(
            Expression(flip_op(expression.operator), [right, left]),
            session,
            model,
            get_model,
        )
    else:
        assert left == Variable("_this")
        if not isinstance(right, model):
            return sql.false()

        if expression.operator not in ("Eq", "Unify"):
            raise UnsupportedError(
                f"Unsupported comparison: {expression}. Models can only be compared"
                " with `=` or `==`"
            )

        primary_keys = [pk.name for pk in inspect(model).primary_key]
        pk_filter = sql.true()
        for key in primary_keys:
            pk_filter = and_filter(
                pk_filter, getattr(model, key) == getattr(right, key)
            )
        return pk_filter


def translate_in(expression, session, model, get_model):
    assert expression.operator == "In"
    left = expression.args[0]
    right = expression.args[1]

    # IN means at least something must be contained in the property.

    # There are two possible types of in operations. In both, the right hand side
    # should be a dot op.

    path = dot_path(right)
    assert path[0] == "_this"
    path = path[1:]
    assert path

    # Partial In: LHS is an expression
    if isinstance(left, Expression):
        return translate_dot(
            path,
            session,
            model,
            functools.partial(emit_subexpression, left, get_model),
        )
    elif isinstance(left, Variable):
        # A variable with no additional constraints
        return translate_dot(
            path,
            session,
            model,
            functools.partial(emit_subexpression, Expression("And", []), get_model),
        )
    else:
        # Contains: LHS is not an expression.
        # TODO (dhatch) Missing check, left type must match type of the target?
        path, field_name = path[:-1], path[-1]
        return translate_dot(
            path, session, model, functools.partial(emit_contains, field_name, left)
        )


def translate_dot(path: Tuple[str, ...], session: Session, model, func: EmitFunction):
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


def emit_compare(field_name, value, operator, session, model):
    """Emit a comparison operation comparing the value of ``field_name`` on ``model`` to ``value``."""
    assert not isinstance(value, Variable), "value is a variable"
    property = getattr(model, field_name)
    return COMPARISONS[operator](property, value)


def emit_subexpression(sub_expression: Expression, get_model, session: Session, model):
    """Emit a sub-expression on ``model``."""
    return translate_expr(sub_expression, session, model, get_model)


def emit_contains(field_name, value, session, model):
    """Emit a contains operation, checking that multi-valued relationship field ``field_name`` contains ``value``."""
    # TODO (dhatch): Could this be valid for fields that are not relationship fields?
    property, model, is_multi_valued = get_relationship(model, field_name)
    assert is_multi_valued

    return property.contains(value)
