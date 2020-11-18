from django.db.models import Q

from polar.expression import Expression
from polar.variable import Variable
from polar.exceptions import UnsupportedError


def partial_to_query_filter(partial, type_name):
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

    q = translate_expr(partial, type_name)
    if q is None:
        return Q()

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


def translate_expr(expr: Expression, type_name: str, **kwargs):
    """Translate a Polar expression to a Django Q object."""
    assert isinstance(expr, Expression), "expected a Polar expression"

    if expr.operator in COMPARISONS:
        return compare_expr(expr, type_name, **kwargs)
    elif expr.operator == "And":
        return and_expr(expr, type_name, **kwargs)
    elif expr.operator == "Isa":
        try:
            assert expr.args[1].tag == type_name
        except (AssertionError, IndexError, AttributeError, TypeError):
            raise UnsupportedError(f"Unimplemented partial isa operation {expr}.")
        return None
    elif expr.operator == "In":
        return in_expr(expr, type_name, **kwargs)
    else:
        raise UnsupportedError(f"Unimplemented partial operator {expr.operator}")


def and_expr(expr: Expression, type_name: str, **kwargs):
    assert expr.operator == "And"
    q = Q()
    for arg in expr.args:
        expr = translate_expr(arg, type_name, **kwargs)
        if expr:
            q = q & expr
    return q


def compare_expr(expr: Expression, _type_name: str, path=[], **kwargs):
    q = Q()
    (left, right) = expr.args
    left_path = dot_op_path(left)
    assert left_path, "this arg should be normalized to LHS"
    return COMPARISONS[expr.operator](q, "__".join(path + left_path), right)


def in_expr(expr: Expression, type_name: str, path=[], **kwargs):
    assert expr.operator == "In"
    q = Q()
    (left, right) = expr.args
    right_path = dot_op_path(right)
    assert right_path, "RHS of in must be a dot lookup"
    right_path = path + right_path

    if isinstance(left, Expression):
        if left.operator == "And":
            # Distribute the expression over the "In".
            return and_expr(left, type_name, path=right_path, **kwargs)
        elif left.operator == "In":
            # Nested in operations.
            return in_expr(left, type_name, path=right_path, **kwargs)
        elif left.operator in COMPARISONS:
            # `tag in post.tags and tag.created_by = user` where `post` is a
            # partial and `user` is a Django instance.
            return compare_expr(left, type_name, path=right_path, **kwargs)
        else:
            assert False, f"Unhandled expression {left}"
    else:
        # `tag in post.tags and user in tag.users` where `post` is a partial
        # and `user` is a Django instance.
        return COMPARISONS["Unify"](q, "__".join(right_path), left)


# TODO (dhatch): Move this helper into base.
def dot_op_path(expr):
    """Get the path components of a lookup.

    The path is returned as a list.

    _this.created_by => ['created_by']
    _this.created_by.username => ['created_by', 'username']

    None is returned if input is not a dot operation.
    """

    if not (isinstance(expr, Expression) and expr.operator == "Dot"):
        return None

    assert len(expr.args) == 2

    if expr.args[0] == Variable("_this"):
        return [expr.args[1]]

    return dot_op_path(expr.args[0]) + [expr.args[1]]
