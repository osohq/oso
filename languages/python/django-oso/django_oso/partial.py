from typing import Tuple, Union
from django.db.models import F, Q, Model
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
# as field > 0 instead
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


# pylint: disable=E1136 # PyCQA/pylint/issues/3882
def get_model_by_path(
    model: Model, path: Union[Tuple[()], Tuple[Variable, ...]]
) -> Model:
    for attr in path:
        model = model._meta.get_field(attr).related_model
    return model


# pylint: disable=E1136 # PyCQA/pylint/issues/3882
def sub_this(
    expr,
) -> Union[Expression, Variable]:
    if isinstance(expr, Variable):
        return Variable("_this")
    else:
        isinstance(expr, Expression) and expr.operator == "Dot"
        return Expression("Dot", [sub_this(expr.args[0]), expr.args[1]])


class FilterBuilder:
    def __init__(self, model: Model, parent=None):
        self.model = model
        self.filter = Q()
        # Map variables to field paths
        self.variables = {}
        # Map of field path to FilterBuilders
        self.subqueries = {}
        self.parent = parent

    def translate_path_to_field(self, path):
        if path[0] == "_this":
            # breakpoint()
            return F("__".join(path[1:]))
        elif path[0] in self.variables:
            breakpoint()
            return F("__".join(self.variables[path[0]] + path[1:]))
        elif self.parent:
            parental_path = self.parent.translate_path_to_field(path)
            breakpoint()
            return OuterRef(parental_path)
        else:
            breakpoint()

    def get_query_from_var(self, var):
        if var in self.variables:
            return self.subqueries[self.variables[var]]
        for subquery in self.subqueries.values():
            query = subquery.get_query_from_var(var)
            if query is not None:
                return query

    def isa_expr(self, expr: Expression):
        assert expr.operator == "Isa"
        (left, right) = expr.args
        left_path = dot_path(left)
        assert left_path[0] == "_this"
        model = get_model_by_path(self.model, left_path[1:])
        constraint_type = apps.get_model(django_model_name(right.tag))
        assert not right.fields, "Unexpected fields in matches expression"
        assert issubclass(
            model, constraint_type
        ), "Inapplicable rule should have been filtered out"
        self.filter &= TRUE_FILTER

    def translate_expr(self, expr: Expression):
        """Translate a Polar expression to a Django Q object."""
        assert isinstance(expr, Expression), "expected a Polar expression"

        # Check if either side of the expression starts with a lookup on
        # a variable. In which case, enter the subquery for that variable
        # instead and proceed as usual.
        if len(expr.args) == 2:
            left, right = expr.args
            left_path = dot_path(left)
            right_path = dot_path(right)

            if left_path and left_path[1:]:
                query = self.get_query_from_var(left_path[0])
                if query:
                    expr.args[0] = sub_this(left)
                    query.translate_expr(expr)
                    return self
            if right_path and right_path[1:]:
                query = self.get_query_from_var(right_path[0])
                if query:
                    expr.args[1] = sub_this(right)
                    query.translate_expr(expr)
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
                breakpoint()
                return

    def compare_expr(self, expr: Expression):
        assert expr.operator in COMPARISONS
        (left, right) = expr.args
        left_path = dot_path(left)
        right_path = dot_path(right)
        if left_path and left_path[1:]:
            self.filter &= COMPARISONS[expr.operator]("__".join(left_path[1:]), right)
        elif right_path and right_path[1:]:
            self.filter &= REFLECTED_COMPARISONS[expr.operator](
                "__".join(right_path[1:]), left
            )
        elif left == Variable("_this"):
            assert self.model is not None
            assert isinstance(right, self.model)
            if expr.operator not in ("Eq", "Unify"):
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
        right_path = dot_path(right)

        if left == "_this" and right_path:
            if right_path[1:]:
                # _this in _this.foo.bar
                # _this in _some_var.foo.bar
                path = self.translate_path_to_field(right_path)
                # path = "__".join(right_path[1:])
                self.filter &= COMPARISONS["Unify"]("pk", path)
            else:
                # _this in _this
                # _this in _some_var
                breakpoint()
        elif isinstance(left, Variable) and right_path:
            if right_path[1:]:
                # var in _this.foo.bar
                # var in other_var.foo.bar

                # Left is a variable => apply constraints to the subquery.
                if left not in self.variables:
                    self.variables[left] = right_path[1:]
                else:
                    breakpoint()
                    # This means we have two paths for the same variable
                    # the subquery will handle the intersection

                # Get the model for the subfield
                model = get_model_by_path(self.model, right_path[1:])
                if right_path[1:] not in self.subqueries:
                    self.subqueries[right_path[1:]] = FilterBuilder(model, parent=self)

                subquery = self.subqueries[right_path[1:]]
                # <var> in <partial>
                # => set up <var> as a new filtered query over the model
                # filtered to the entries of right_path
                path = self.translate_path_to_field(right_path)
                if isinstance(path, F):
                    subquery.filter &= Q(pk=OuterRef(path.name))
                else:
                    subquery.filter &= Q(pk=OuterRef(path))
                # Maybe redundant, but want to be sure
                self.subqueries[right_path[1:]] = subquery
            else:
                # var in _this
                # var in other_var
                breakpoint()
        else:
            # <value> in <partial>
            self.filter &= COMPARISONS["Unify"]("__".join(right_path[1:]), left)

    def not_expr(self, expr: Expression):
        assert expr.operator == "Not"
        assert expr.args[0].operator == "Isa"
        fb = FilterBuilder(self.model, parent=self.parent)
        fb.translate_expr(expr.args[0])
        self.filter &= ~fb.finish()

    def finish(self):
        # For every subquery, finish off by checking these are non-empty
        for _var, path in self.variables.items():
            subq = self.subqueries[path]
            filtered = subq.model.objects.filter(subq.finish()).values("pk")
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
