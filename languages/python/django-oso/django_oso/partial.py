from typing import Tuple, Union
from django.db.models import F, Q, Model
from django.apps import apps
from django.db.models.expressions import Exists, OuterRef

from polar.expression import Expression
from polar.exceptions import UnsupportedError
from polar.partial import dot_path
from oso import Variable

from .oso import django_model_name

# A query filter that is always true, but can still be combined
# with other filters to narrow the scope.
# See: https://forum.djangoproject.com/t/improving-q-objects-with-true-false-and-none/851/1
# for more details
TRUE_FILTER = ~Q(pk__in=[])

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
def reflect_expr(expr: Expression):
    assert expr.operator in COMPARISONS
    reflections = {
        "Gt": "Lt",
        "Geq": "Leq",
        "Lt": "Gt",
        "Leq": "Geq",
    }
    left, right = expr.args
    op = expr.operator
    return Expression(reflections.get(op, op), [right, left])


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
        self.filter = TRUE_FILTER
        # Map variables to subquery
        self.variables = {}
        self.parent = parent

    def add_filter(self, filter):
        if self.filter == TRUE_FILTER:
            self.filter = filter
        else:
            self.filter &= filter

    def translate_path_to_field(self, path):
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
        for subquery in self.variables.values():
            query = subquery.get_query_from_var(var)
            if query is not None:
                return query

    def isa_expr(self, expr: Expression):
        assert expr.operator == "Isa"
        (left, right) = expr.args
        left_path = dot_path(left)
        assert left_path[0] == self.name
        root = self.get_query_from_var(left_path[0])
        model = get_model_by_path(root.model, left_path[1:])
        ty = apps.get_model(django_model_name(right.tag))
        assert not right.fields, "Unexpected fields in matches expression"
        assert issubclass(model, ty), "Inapplicable rule should have been filtered out"

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

    def compare_expr(self, expr: Expression):
        assert expr.operator in COMPARISONS
        (left, right) = expr.args
        left_path = dot_path(left)
        right_path = dot_path(right)

        # Normalize partial to LHS.
        if right_path:
            expr = reflect_expr(expr)
            left, right = right, left
            left_path, right_path = right_path, left_path

        left_field = "__".join(left_path[1:]) if left_path[1:] else "pk"

        if left_path and right_path:
            raise UnsupportedError(f"Unsupported partial expression: {expr}")
            # compare partials
            # self.add_filter(COMPARISONS[expr.operator](
            #     left_field, self.translate_path_to_field(right_path)
            # ))
        else:
            # partial cmp grounded
            assert left_path
            if isinstance(right, Model):
                right = right.pk
            self.add_filter(COMPARISONS[expr.operator](left_field, right))

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
                self.add_filter(COMPARISONS["Unify"]("pk", path))
            else:
                # _this in _this
                # _this in _some_var
                raise UnsupportedError(f"Unsupported partial expression: {expr}")
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
                    subquery_path = right_path[1:]
                    model = get_model_by_path(base_query.model, subquery_path)
                    base_query.variables[left] = FilterBuilder(
                        model, parent=base_query, name=left
                    )
                else:
                    # This means we have two paths for the same variable
                    # the subquery will handle the intersection
                    pass

                # Get the model for the subfield

                subquery = base_query.variables[left]
                # <var> in <partial>
                # => set up <var> as a new filtered query over the model
                # filtered to the entries of right_path
                path = base_query.translate_path_to_field(right_path)
                field = OuterRef(path.name) if isinstance(path, F) else OuterRef(path)
                subquery.filter &= COMPARISONS["Unify"]("pk", field)
                # Maybe redundant, but want to be sure
                base_query.variables[left] = subquery
            else:
                # var in _this
                # var in other_var
                raise UnsupportedError(f"Unsupported partial expression: {expr}")
        else:
            # <value> in <partial>
            self.add_filter(COMPARISONS["Unify"]("__".join(right_path[1:]), left))

    def not_expr(self, expr: Expression):
        assert expr.operator == "Not"
        assert expr.args[0].operator == "Isa"
        fb = FilterBuilder(self.model, parent=self.parent)
        fb.translate_expr(expr.args[0])
        self.add_filter(~fb.finish())

    def finish(self):
        """For every subquery, construct a filter to make sure the result set is non-empty"""
        if len(self.variables) == 0:
            return self.filter
        objects = self.model.objects.all()
        for subq in self.variables.values():
            filtered = subq.model.objects.filter(subq.finish()).values("pk")
            exists = Exists(filtered)
            name = f"{self.name}__exists"
            # https://docs.djangoproject.com/en/2.2/ref/models/expressions/#filtering-on-a-subquery-expression
            objects = objects.annotate(**{name: exists}).filter(**{name: True})
        self.add_filter(Q(pk__in=objects.values("pk")))
        return self.filter


def partial_to_query_filter(partial: Expression, model: Model):
    """
    Convert a partial expression to a django query ``Q`` object.

    Example expression structure::

        Expression(And, [
            Expression(Isa, [
                Variable('_this'),
                Pattern(test_app::Post, {})]),
            Expression(Unify, [
                False,
                Expression(Dot, [
                    Variable('_this'),
                    'is_private'])])])

    Output::

        Q(is_private=False)
    """
    fb = FilterBuilder(model)
    fb.translate_expr(partial)
    return fb.finish()
