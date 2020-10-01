from django.db.models import Q

from polar.expression import Expression
from polar.variable import Variable


def partial_to_query_filter(partial, type_name):
    """
    Expression(And, [
        Expression(Isa, [
            Variable('_this'),
            Pattern(test_app::Post, {})]),
        Expression(Isa, [
            Variable('_this'),
            Pattern(test_app::Post, {})]),
        Expression(Unify, [
            False,
            Expression(
                Dot, [
                    Variable('_this'),
                    'is_private'])])])
    """

    q = and_expr(partial, type_name)
    return q


COMPARISONS = {
    "Unify": lambda q, f, v: Q(**{f: v}),
    "Eq": lambda q, f, v: Q(**{f: v}),
    "Neq": lambda q, f, v: ~Q(**{f: v}),
    "Geq": lambda q, f, v: Q(**{f"{f}__gte": v}),
    "Gt": lambda q, f, v: Q(**{f"{f}__gt": v}),
    "Leq": lambda q, f, v: Q(**{f"{f}__leq": v}),
    "Lt": lambda q, f, v: Q(**{f"{f}__lt": v}),
}


def and_expr(expr, type_name):
    q = Q()

    assert expr.operator == "And"
    for expression in expr.args:
        assert isinstance(expression, Expression)
        if expression.operator in COMPARISONS:
            q = q & compare_expr(expression, type_name)
        elif expression.operator == "And":
            q = q & and_expr(expression, type_name)
        elif expression.operator == "Or":
            assert False
        elif expression.operator == "Isa":
            assert expression.args[1].tag == type_name
        else:
            raise Exception(f"Unexpected operator {expression.operator}")

    return q


def dot_op_field(expr):
    """Get the field from dot op ``expr`` or return ``False``."""
    return (
        isinstance(expr, Expression)
        and expr.operator == "Dot"
        and isinstance(expr.args[0], Variable)
        and expr.args[0] == Variable("_this")
        and expr.args[1]
    )


def compare_expr(expr, type_name):
    q = Q()

    assert expr.operator in COMPARISONS
    left = expr.args[0]
    right = expr.args[1]

    if dot_op_field(left):
        field = dot_op_field(left)
        value = right
        return COMPARISONS[expr.operator](q, field, value)
    else:
        field = dot_op_field(right)
        assert field
        value = left
        return COMPARISONS[expr.operator](q, field, value)
