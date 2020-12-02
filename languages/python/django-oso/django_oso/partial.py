from django.db.models import Q, Model
from django.apps import apps

from polar.expression import Expression
from polar.exceptions import UnsupportedError
from polar.partial import dot_path

from .oso import django_model_name


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

    q = translate_expr(partial, model, **kwargs)
    if q is None:
        return Q()

    return q


COMPARISONS = {
    "Unify": lambda f, v: Q(**{f: v}),
    "Eq": lambda f, v: Q(**{f: v}),
    "Neq": lambda f, v: ~Q(**{f: v}),
    "Geq": lambda f, v: Q(**{f"{f}__gte": v}),
    "Gt": lambda f, v: Q(**{f"{f}__gt": v}),
    "Leq": lambda f, v: Q(**{f"{f}__leq": v}),
    "Lt": lambda f, v: Q(**{f"{f}__lt": v}),
    "In": lambda f, v: Q(**{f"{f}__in": v}),
}


def translate_expr(expr: Expression, model: Model, **kwargs):
    """Translate a Polar expression to a Django Q object."""
    assert isinstance(expr, Expression), "expected a Polar expression"

    if expr.operator == "And":
        return and_expr(expr, model, **kwargs)
    elif expr.operator == "Isa":
        return isa_expr(expr, model, **kwargs)
    elif expr.operator == "In":
        return in_expr(expr, model, **kwargs)
    elif expr.operator in COMPARISONS:
        return compare_expr(expr, model, **kwargs)
    else:
        raise UnsupportedError(f"Unimplemented partial operator {expr.operator}")


def isa_expr(expr: Expression, model: Model, **kwargs):
    (left, right) = expr.args
    for attr in dot_path(left):
        model = getattr(model, attr).field.related_model
    constraint_type = apps.get_model(django_model_name(right.tag))
    if not issubclass(model, constraint_type):
        # Always false.
        return Q(pk__in=[])
    else:
        # Always true.
        return None


def and_expr(expr: Expression, model: Model, **kwargs):
    assert expr.operator == "And"
    q = Q()
    for arg in expr.args:
        expr = translate_expr(arg, model, **kwargs)
        if expr:
            q = q & expr
    return q


def compare_expr(expr: Expression, _model: Model, path=(), **kwargs):
    (left, right) = expr.args
    left_path = dot_path(left)
    if left_path:
        return COMPARISONS[expr.operator]("__".join(path + left_path), right)
    else:
        if isinstance(right, Model):
            right = right.pk
        else:
            raise UnsupportedError(f"Unsupported comparison: {expr}")
        return COMPARISONS[expr.operator]("__".join(path + ("pk",)), right)


def in_expr(expr: Expression, model: Model, path=(), **kwargs):
    assert expr.operator == "In"
    (left, right) = expr.args
    right_path = dot_path(right)
    assert right_path, "RHS of in must be a dot lookup"
    right_path = path + right_path

    if isinstance(left, Expression):
        if left.operator == "And" and not left.args:
            from django.db.models import Count, Subquery

            count = Count("__".join(right_path))
            filter = COMPARISONS["Gt"]("__".join(right_path + ("count",)), 0)
            subquery = Subquery(
                model.objects.annotate(count).filter(filter).values("pk")
            )

            return COMPARISONS["In"]("pk", subquery)
        else:
            return translate_expr(left, model, path=right_path, **kwargs)
    else:
        return COMPARISONS["Unify"]("__".join(right_path), left)
