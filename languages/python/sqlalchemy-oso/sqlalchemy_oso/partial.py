"""Translate Oso Expressions into SQLAlchemy Expressions.

This module translates between Oso Expressions that are returned from queries
with partial variables into SQLAlchemy constraints.

The entrypoint is :py:func:`partial_to_filter`. The translation is written as a
recursive mapping operation. We do a traversal of the expression tree, using the
``translate_*`` functions to map each node of the Oso expression tree to a
SQLAlchemy expression.

Translation functions
=====================

These functions accept as input:

- ``expression``: an :py:class:`polar.expression.Expression` instance returned
  by the query. The expression must be translated by
  :py:func:`sqlalchemy_oso.preprocess.preprocess`.
- ``session``: The :py:class:`sqlalchemy.orm.Session` session object to
  translate for.
- ``model``: The model class that this expression is constraining.
- ``get_model``: A callable that returns a SQLAlchemy model type corresponding
  with a Polar type tag.

Expression structure
--------------------

The translation functions operate over expressions that constrain a single
variable, named ``_this`` which corresponds to the ``model`` pararmeter.
Constraints on a to-many relationship (expressed in Polar like ``tag in
post.tags and tag.id = 1``) are represented as a subexpression. The Polar::

    allow(_, _, post) if
        post.id = 1 and tag in post.tags and
        tag.id = 2 and
        tag.is_public;

Would be represented as the expression::

    _this.id = 1 and (_this.id = 2 and _this.is_public= true) in post.tags

- :py:func:`translate_expr`: Translate an expression.
- :py:func:`translate_and`: Translate an and operation
- :py:func:`translate_compare`: Translate a comparison operation (=, <, etc.)
- :py:func:`translate_in`: Translate an in operation.
- :py:func:`translate_isa`: Translate an isa.
- :py:func:`translate_dot`: Translate a dot operation.


Emit functions
==============

The functions :py:func:`emit_compare`, :py:func:`emit_contains`, and
:py:func:`emit_subexpression` are used by :py:func:`translate_dot` to aid in
producing SQLAlchemy expressions over dot operations.  More information on this
in the :py:func:`translate_dot` documentation string.

Examples in module documentation
================================

Throughout the documentation of this module, we will refer to examples
corresponding to the models declared in ``tests/models.py``.

When recursive translation is applied to an operation, the notation ``t(?)`` is
used.
"""

import functools
from typing import Any, Callable, Tuple

from polar.exceptions import PolarRuntimeError, UnsupportedError
from polar.expression import Expression
from polar.partial import dot_path
from polar.variable import Variable
from sqlalchemy import inspect
from sqlalchemy.orm import RelationshipProperty
from sqlalchemy.orm.session import Session
from sqlalchemy.sql import expression as sql
from sqlalchemy.sql.elements import True_

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


def flip_op(operator: str) -> str:
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
    if isinstance(current, bool):
        current = sql.true() if current else sql.false()
    if isinstance(current, True_):
        return new
    else:
        return current & new


def partial_to_filter(expression: Expression, session: Session, model, get_model):
    """Convert constraints in ``partial`` to a filter over ``model`` that should be applied to query."""
    expression = preprocess(expression)
    return translate_expr(expression, session, model, get_model)


def translate_expr(expression: Expression, session: Session, model, get_model):
    """Translate an expression into a SQLAlchemy expression.

    Accepts any type of expression. Entrypoint to the translation functions."""
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
    """Translate a Polar AND into a SQLAlchemy AND.

    Empty and is true: () => sql.true()
    Single argument: op1 => t(op1)
    > 1 argument: op1 and op2 and op3 => t(op1) & t(op2) & t(op3)
    """
    assert expression.operator == "And"
    expr = sql.true()
    for expression in expression.args:
        translated = translate_expr(expression, session, model, get_model)
        expr = and_filter(expr, translated)

    return expr


def translate_isa(expression: Expression, session: Session, model, get_model):
    """Translate an Isa operation. (``matches`` keyword)

    Check that the field on the left hand side matches the type on the right.

    ``isa`` operations with fields are not supported and throw.

    If the type matches, ``sql.true()`` is returned. If the type doesn't match,
    ``sql.false()`` is returned.

    So for example::

        allow(_, _, x) if x matches Tag;

    would translate to sql.false() (no rows match) when ``x`` is of type Post,
    but would translate to ``sql.true()`` when ``x`` is of type Tag.

    _this matches Type => sql.true() if Type ==  model else sql.false()
    _this.bar matches Type => sql.true() if typeof(model, "bar") == Type

    Where typeof gives the type of the "bar" property of model.
    """
    assert expression.operator == "Isa"
    left, right = expression.args
    left_path = dot_path(left)
    if left_path:
        assert left_path[0] == Variable("_this")
        left_path = left_path[1:]  # Drop _this.
        for field_name in left_path:
            _, model, __ = get_relationship(model, field_name)

    assert not right.fields, "Unexpected fields in isa expression"
    constraint_type = get_model(right.tag)
    model_type = inspect(model, raiseerr=True).class_
    return (
        sql.true()
        if issubclass(model_type, constraint_type) or isinstance(left, constraint_type)
        else sql.false()
    )


def translate_compare(expression: Expression, session: Session, model, get_model):
    """Translate a binary comparison operation.

    Operators are listed in ``COMPARISONS``.

    Either the left or right argument may contain a path. Paths for both
    arguments (i.e. post.name = post.body) are not supported currently.

    Also handle unification of _this with an instance of the same type as _this. E.g., _this = ?
    where ? is an instance of the same type as _this.

    _this.path.(path1)+.tail OP val => Model.path.(path1)+.has(Target.tail OP val)
    val OP _this.path.(path1)+.tail => Model.path.(path1)+.has(Target.tail OP mirror(OP) val)
    _this = val => model.pk1 = val.pk1 and model.pk2 = val.pk2

    Where Target is the type that the dot path refers to and mirror flips an
    operation.
    """
    (left, right) = expression.args
    left_path = dot_path(left)
    right_path = dot_path(right)

    # Dot operation is on the left hand side
    if left_path[1:]:
        assert left_path[0] == Variable("_this")
        if right_path:
            raise PolarRuntimeError(
                "Invalid comparison in policy. This may be caused by comparing the "
                + "foreign key field rather than the relationship property"
            )
        path, field_name = left_path[1:-1], left_path[-1]
        return translate_dot(
            path,
            session,
            model,
            functools.partial(emit_compare, field_name, right, expression.operator),
        )
    # Dot operation is on right
    elif right_path and right_path[0] == "_this":
        return translate_compare(
            Expression(flip_op(expression.operator), [right, left]),
            session,
            model,
            get_model,
        )
    # this = other no dot operation, throws if it's not of the form _this = other other same type as
    # this
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


def translate_in(expression: Expression, session: Session, model, get_model):
    """Translate the in operator.

    Relationship contains at least one value that matches expr.
    (expr) in _this.path.(path1)+ => Model.path.(path1)+.any(t(expr))

    relationship at least 1 value with no constraints:
    () in _this.path.(path1)+ => Model.path.(path1)+.any(sql.true())

    relationship contains val
    val in _this.path.(path1)+ => Model.path.(path1)+.contains(val)
    """

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
    """Translate an operation over a path.

    Used to translate comparison operations over paths, and in operations.

    Walks relationship properties on ``model`` using ``path``, ending by calling
    ``func`` with ``session`` and the ``model`` of the last field as positional
    arguments.

    This results in adding an ``EXISTS (SELECT 1 FROM related_table WHERE ...)`` to
    the expression, as documented in the SQLAlchemy documentation for ``has``
    and ``any``. The ``...`` will either be the next segment of the dot path, or
    the result of ``func``.
    """

    if len(path) == 0:
        return func(session, model)
    else:
        property, model, is_multi_valued = get_relationship(model, path[0])
        if not is_multi_valued:
            return property.has(translate_dot(path[1:], session, model, func))
        else:
            return property.any(translate_dot(path[1:], session, model, func))


def get_relationship(model: type, field_name: str):
    """Get the property object for field on model. field must be a relationship field.

    :returns: (property, model, is_multi_valued)
    """
    property = getattr(model, field_name)
    assert isinstance(property.property, RelationshipProperty)
    relationship = property.property
    model = property.entity.class_

    return (property, model, relationship.uselist)


def emit_compare(field_name: str, value, operator, session: Session, model):
    """Emit a comparison operation comparing the value of ``field_name`` on ``model`` to ``value``."""
    assert not isinstance(value, Variable), "value is a variable"
    property = getattr(model, field_name)
    return COMPARISONS[operator](property, value)


def emit_subexpression(sub_expression: Expression, get_model, session: Session, model):
    """Emit a sub-expression on ``model``."""
    return translate_expr(sub_expression, session, model, get_model)


def emit_contains(field_name: str, value, session: Session, model):
    """Emit a contains operation, checking that multi-valued relationship field ``field_name`` contains ``value``."""
    # TODO (dhatch): Could this be valid for fields that are not relationship fields?
    property, model, is_multi_valued = get_relationship(model, field_name)
    assert is_multi_valued

    return property.contains(value)
