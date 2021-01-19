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
    "Leq": lambda f, v: Q(**{f"{f}__lte": v}),
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


class FilterBuilder:
    def __init__(self, model: Model, name="_this", parent=None):
        self.name = name
        self.model = model
        self.filter = Q()
        # Map variables to field paths
        self.variables = {}
        # Map of field path to FilterBuilders
        self.subqueries = {}
        self.parent = parent

    def translate_path_to_field(self, path, outer=False):
        if path[0] == self.name:
            path = "__".join(path[1:])
            return F(path)
        elif path[0] in self.variables:
            return F("__".join(self.variables[path[0]] + path[1:]))
        elif self.parent:
            parental_path = self.parent.translate_path_to_field(path)
            if isinstance(parental_path, F):
                parental_path = parental_path.name
            return OuterRef(parental_path)
        else:
            raise Exception(f"{path} cannot be handled")

    def get_query_from_var(self, var):
        if var == self.name:
            return self
        elif var in self.variables:
            return self.subqueries[self.variables[var]]
        for subquery in self.subqueries.values():
            query = subquery.get_query_from_var(var)
            if query is not None:
                return query

    def isa_expr(self, expr: Expression):
        assert expr.operator == "Isa"
        (left, right) = expr.args
        left_path = dot_path(left)
        # assert left_path[0] == "_this"
        root = self.get_query_from_var(left_path[0])
        model = get_model_by_path(root.model, left_path[1:])
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

            if left_path:
                query = self.get_query_from_var(left_path[0])
                if query and query != self:
                    query.translate_expr(expr)
                    return self
            if right_path:
                query = self.get_query_from_var(right_path[0])
                if query and query != self:
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
        left_field = "__".join(left_path[1:]) if left_path[1:] else "pk"
        right_field = "__".join(right_path[1:]) if right_path[1:] else "pk"

        if left_path and right_path:
            # compare partials
            if left_path[0] == "_this":
                self.filter &= COMPARISONS[expr.operator](
                    left_field, self.translate_path_to_field(right_path)
                )
            else:
                assert right_path[0] == "_this"
                self.filter &= REFLECTED_COMPARISONS[expr.operator](
                    right_field, self.translate_path_to_field(left_path)
                )
        elif left_path:
            # partial cmp grounded
            if isinstance(right, Model):
                right = right.pk
            self.filter &= COMPARISONS[expr.operator](left_field, right)
        elif right_path:
            # grounded cmp partial
            if isinstance(left, Model):
                left = left.pk
            self.filter &= REFLECTED_COMPARISONS[expr.operator](right_field, left)
        else:
            # grounded cmp grounded???
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

                # get the base query for the RHS of the `in`
                root = self
                while root.parent:
                    root = root.parent
                base_query = root.get_query_from_var(right_path[0]) or root

                # Left is a variable => apply constraints to the subquery.
                if left not in base_query.variables:
                    base_query.variables[left] = right_path[1:]
                else:
                    # This means we have two paths for the same variable
                    # the subquery will handle the intersection
                    pass

                # Get the model for the subfield
                model = get_model_by_path(base_query.model, right_path[1:])
                if right_path[1:] not in base_query.subqueries:
                    base_query.subqueries[right_path[1:]] = FilterBuilder(
                        model, parent=base_query, name=left
                    )

                subquery = base_query.subqueries[right_path[1:]]
                # <var> in <partial>
                # => set up <var> as a new filtered query over the model
                # filtered to the entries of right_path
                path = base_query.translate_path_to_field(right_path)
                field = OuterRef(path.name) if isinstance(path, F) else OuterRef(path)
                subquery.filter &= COMPARISONS["Unify"]("pk", field)
                # Maybe redundant, but want to be sure
                base_query.subqueries[right_path[1:]] = subquery
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
