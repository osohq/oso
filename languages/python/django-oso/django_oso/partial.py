from django.db.models import Q, Model, Count, Subquery
from django.apps import apps

from polar.expression import Expression
from polar.exceptions import UnsupportedError
from polar.partial import dot_path
from oso import Variable

from .oso import django_model_name


TRUE_FILTER = ~Q(pk__in=[])
FALSE_FILTER = Q(pk__in=[])

COMPARISONS = {
    "Unify": lambda f, v: Q(**{f: v}),
    "Eq": lambda f, v: Q(**{f: v}),
    "Neq": lambda f, v: ~Q(**{f: v}),
    "Geq": lambda f, v: Q(**{f"{f}__gte": v}),
    "Gt": lambda f, v: Q(**{f"{f}__gt": v}),
    "Leq": lambda f, v: Q(**{f"{f}__leq": v}),
    "Lt": lambda f, v: Q(**{f"{f}__lt": v}),
}


def contained_in(f, v):
    return Q(**{f"{f}__in": v})


def partial_to_query_filter(partial: Expression, model: Model, **kwargs):
    """
    Convert a partial expression to a django query ``Q`` object.

    Example expression structure::

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

    Output::

        Q(is_private=False)
    """

    return translate_expr(partial, model, **kwargs)


def translate_expr(expr: Expression, model: Model, **kwargs):
    """Translate a Polar expression to a Django Q object."""
    assert isinstance(expr, Expression), "expected a Polar expression"

    if expr.operator in COMPARISONS:
        return compare_expr(expr, model, **kwargs)
    elif expr.operator == "And":
        return and_expr(expr, model, **kwargs)
    elif expr.operator == "Isa":
        return isa_expr(expr, model, **kwargs)
    elif expr.operator == "In":
        return in_expr(expr, model, **kwargs)
    elif expr.operator == "Not":
        return not_expr(expr, model, **kwargs)
    else:
        raise UnsupportedError(f"Unsupported partial expression: {expr}")


def isa_expr(expr: Expression, model: Model, **kwargs):
    assert expr.operator == "Isa"
    (left, right) = expr.args
    for attr in dot_path(left):
        model = model._meta.get_field(attr).related_model
    constraint_type = apps.get_model(django_model_name(right.tag))
    assert not right.fields, "Unexpected fields in matches expression"
    return TRUE_FILTER if issubclass(model, constraint_type) else FALSE_FILTER


def and_expr(expr: Expression, model: Model, **kwargs):
    assert expr.operator == "And"
    q = Q()
    for arg in expr.args:
        expr = translate_expr(arg, model, **kwargs)
        # TODO: Remove once we can perform method selection in the presence of partials.
        # Short-circuit: if any expr is false, the whole AND is false.
        if expr == FALSE_FILTER:
            return FALSE_FILTER
        q &= expr
    return q


def compare_expr(expr: Expression, model: Model, path=(), **kwargs):
    assert expr.operator in COMPARISONS
    (left, right) = expr.args
    left_path = dot_path(left)
    if left_path:
        return COMPARISONS[expr.operator]("__".join(path + left_path), right)
    else:
        assert left == Variable("_this")
        if not isinstance(right, model):
            return FALSE_FILTER

        if expr.operator not in ("Eq", "Unify"):
            raise UnsupportedError(
                f"Unsupported comparison: {expr}. Models can only be compared"
                " with `=` or `==`"
            )

        return COMPARISONS[expr.operator]("__".join(path + ("pk",)), right.pk)


def in_expr(expr: Expression, model: Model, path=(), **kwargs):
    assert expr.operator == "In"
    (left, right) = expr.args
    right_path = dot_path(right)
    assert right_path, "RHS of in must be a dot lookup"
    right_path = path + right_path

    if isinstance(left, Expression):
        if left.operator == "And" and not left.args:
            # An unconstrained partial is in a list if the list is non-empty.
            count = Count("__".join(right_path))
            filter = COMPARISONS["Gt"]("__".join(right_path + ("count",)), 0)
            subquery = Subquery(
                model.objects.annotate(count).filter(filter).values("pk")
            )

            return contained_in("pk", subquery)
        else:
            return translate_expr(left, model, path=right_path, **kwargs)
    else:
        return COMPARISONS["Unify"]("__".join(right_path), left)


def not_expr(expr: Expression, model: Model, **kwargs):
    assert expr.operator == "Not"
    assert expr.args[0].operator == "Isa"
    return ~translate_expr(expr.args[0], model, **kwargs)
