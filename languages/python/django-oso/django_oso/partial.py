from typing import List, Union
from django.db.models import F, Q, Model, Count, Subquery
from django.apps import apps
from django.db.models.expressions import Exists, OuterRef

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

# So that 0 < field can be written
# as field < 0 instead
REFLECTED_COMPARISONS = {
    "Unify": COMPARISONS["Unify"],
    "Eq": COMPARISONS["Eq"],
    "Neq": COMPARISONS["Neq"],
    "Lt": COMPARISONS["Gt"],
    "Leq": COMPARISONS["Geq"],
    "Gt": COMPARISONS["Lt"],
    "Geq": COMPARISONS["Leq"],
}


# TODO: put in comparisons dict above?
def contained_in(f, v):
    return Q(**{f"{f}__in": v})


class FilterBuilder:
    def __init__(self, model: Model):
        self.model = model
        self.filter = Q()
        # Map variables to field paths
        self.variables = {}
        # Map of field path to FilterBuilders
        self.subqueries = {}

    def isa_expr(self, expr: Expression):
        assert expr.operator == "Isa"
        (left, right) = expr.args
        model = self.get_model_by_path(dot_path(left))
        constraint_type = apps.get_model(django_model_name(right.tag))
        assert not right.fields, "Unexpected fields in matches expression"
        self.filter &= (
            TRUE_FILTER if issubclass(model, constraint_type) else FALSE_FILTER
        )

    def get_model_by_path(self, path: List[str]):
        model = self.model
        for attr in path:
            model = model._meta.get_field(attr).related_model
        return model

    # def translate_path(self, arg: Union[Expression, Variable]):
    #     """Convert arg to a path, looking up variables where necessary.
    #     Returns None if arg is not a path-like
    #     """
    #     # return dot_path(arg)
    #     if isinstance(arg, Variable) and arg in self.variables:
    #         path = self.variables[arg]
    #         return path
    #     elif isinstance(arg, Expression) and arg.operator == "Dot":
    #         path = dot_path(arg)
    #         translated = []
    #         for segment in path:
    #             tp = dot_path(segment)
    #             if tp:
    #                 translated += tp
    #             else:
    #                 translated.append(segment)
    #         return translated
    #     else:
    #         # This is not a path-like object
    #         return None

    def translate_expr(self, expr: Expression):
        """Translate a Polar expression to a Django Q object."""
        assert isinstance(expr, Expression), "expected a Polar expression"

        if len(expr.args) == 2:
            left, right = expr.args
            left_path = dot_path(left)
            right_path = dot_path(right)
            if (
                isinstance(left, Expression)
                and left_path
                and isinstance(left_path[0], Variable)
            ):
                var_path = self.variables[left_path[0]]
                left.args[0] = Variable("_this")
                expr.args[0] = left
                self.subqueries[var_path].translate_expr(expr)
                return self
            if (
                isinstance(right, Expression)
                and right_path
                and isinstance(right_path[0], Variable)
            ):
                var_path = self.variables[right_path[0]]
                right.args[0] = Variable("_this")
                expr.args[1] = right
                self.subqueries[var_path].translate_expr(expr)
                return self

        if expr.operator in COMPARISONS:
            self.compare_expr(expr)
        elif expr.operator == "And":
            self.and_expr(expr)
        elif expr.operator == "Isa":
            self.isa_expr(expr)
        elif expr.operator == "In":
            self.in_expr(expr)
        elif expr.operator == "Not":
            self.not_expr(expr)
        else:
            raise UnsupportedError(f"Unsupported partial expression: {expr}")
        return self

    def and_expr(self, expr: Expression):
        assert expr.operator == "And"
        for arg in expr.args:
            self.translate_expr(arg)
            # TODO: Remove once we can perform method selection in the presence of partials.
            # Short-circuit: if any expr is false, the whole AND is false.
            if self.filter == FALSE_FILTER:
                return

    def compare_expr(self, expr: Expression):
        assert expr.operator in COMPARISONS
        (left, right) = expr.args
        left_path = dot_path(left)
        right_path = dot_path(right)
        if left_path:
            if isinstance(left_path[0], Variable):
                path = self.variables[left_path[0]]
                subq = self.subqueries[path]
                subq.filter &= COMPARISONS[expr.operator](
                    "__".join(left_path[1:]), right
                )
            else:
                self.filter &= COMPARISONS[expr.operator]("__".join(left_path), right)
        elif right_path:
            if isinstance(right_path[0], Variable):
                path = self.variables[right_path[0]]
                subq = self.subqueries[path]
                subq.filter &= REFLECTED_COMPARISONS[expr.operator](
                    "__".join(right_path[1:]), left
                )
            else:
                self.filter &= REFLECTED_COMPARISONS[expr.operator](
                    "__".join(right_path), left
                )
        elif left == Variable("_this"):
            if self.model is None:
                self.filter &= FALSE_FILTER
            elif not isinstance(right, self.model):
                self.filter &= FALSE_FILTER
            elif expr.operator not in ("Eq", "Unify"):
                raise UnsupportedError(
                    f"Unsupported comparison: {expr}. Models can only be compared"
                    " with `=` or `==`"
                )
            else:
                self.filter &= COMPARISONS[expr.operator]("pk", right.pk)
        elif right == Variable("_this"):
            breakpoint()
        else:
            breakpoint()

    def in_expr(self, expr: Expression):
        assert expr.operator == "In"
        (left, right) = expr.args
        # left_path = dot_path(left)
        right_path = dot_path(right)

        if left == "_this":
            self.filter &= Q(pk__in=right)

        if isinstance(left, Variable) and isinstance(right, Expression):
            # left is a variable => apply constraints to the
            assert (
                right_path
            ), "constraint of the form <var> in <partial> but the right hand side is not a partial"
            right_path = tuple(right_path)
            if left not in self.variables:
                self.variables[left] = right_path
            else:
                breakpoint()
                # This means we have two paths for the same variable
                # the subquery will handle the intersection

            # Get the model for the subfield
            model = self.get_model_by_path(right_path)
            if right_path not in self.subqueries:
                self.subqueries[right_path] = FilterBuilder(model)

            subquery = self.subqueries[right_path]
            # <var> in <partial>
            # => set up <var> as a new filtered query over the model
            # filtered to the entries of right_path
            subquery.filter &= Q(pk=OuterRef("__".join(right_path)))
            # Maybe redundant, but want to be sure
            self.subqueries[right_path] = subquery
        elif isinstance(left, Expression) and isinstance(right, Expression):
            # <partial> in <partial>
            breakpoint()
        elif isinstance(right, Expression) and right_path:
            # <value> in <partial>
            self.filter &= COMPARISONS["Unify"]("__".join(right_path), left)
        else:
            breakpoint()

    # if isinstance(left, Expression):
    #     if left.operator == "And" and not left.args:
    #         # An unconstrained partial is in a list if the list is non-empty.
    #         count = Count("__".join(right_path))
    #         filter = COMPARISONS["Gt"]("__".join(right_path + ("count",)), 0)
    #         subquery = Subquery(
    #             model.objects.annotate(count).filter(filter).values("pk")
    #         )
    #
    #         return contained_in("pk", subquery)
    #     else:
    #         return translate_expr(left, model, path=right_path)
    # else:
    #     return COMPARISONS["Unify"]("__".join(right_path), left)

    def not_expr(self, expr: Expression):
        assert expr.operator == "Not"
        assert expr.args[0].operator == "Isa"
        fb = FilterBuilder(self.model)
        fb.translate_expr(expr.args[0])
        self.filter &= ~fb.finish()

    def finish(self):
        # For every subquery, finish off by checking these are non-empty
        for _var, path in self.variables.items():
            subq = self.subqueries[path]
            filtered = subq.model.objects.filter(subq.filter)
            exists = Exists(filtered)
            self.filter = exists & self.filter  # This _has_ to be this way around
        return self.filter


def partial_to_query_filter(partial: Expression, model: Model):
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
    fb = FilterBuilder(model)
    fb.translate_expr(partial)
    return fb.finish()
