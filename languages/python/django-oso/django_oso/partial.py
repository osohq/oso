from typing import Tuple, Union
from django.db.models import F, Q, Model
from django.apps import apps

from polar.expression import Expression
from polar.exceptions import UnsupportedError
from polar.partial import dot_path
from oso import Variable

from .oso import django_model_name


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
    def __init__(self, model: Model, name="_this", parent=None, path=()):
        self.name = name
        self.model = model
        self.filter = Q()
        # Map variables to subquery
        self.variables = {}
        self.parent = parent
        self.path = path

    def root(self):
        root = self
        while root.parent:
            root = root.parent
        return root

    def move_to_subquery(self, var):
        """Move `self` to a subquery on the query represented by `var`
        This is effectively making `self` a subdependency of `var`.
        """
        root = self.root()
        other = root.get_query_from_var(var)
        assert other, "expected a query at this point"
        if other != self.parent:
            del self.parent.variables[self.name]
            self.parent = other
            other.variables[self.name] = self

    def translate_path_to_field(self, path):
        f = self.root()._translate_path_to_field(path)
        if not f:
            raise UnsupportedError(f"{path} cannot be converted")
        return f

    def _translate_path_to_field(self, path):
        if len(path) == 1:
            # all we have is a variable (e.g. _this)
            # so we want to use the pk to compare
            return self._translate_path_to_field(path + ("pk",))
        if path[0] == self.name:
            path = "__".join(self.path + path[1:])
            return path

        for q in self.variables.values():
            f = q._translate_path_to_field(path)
            if f:
                return f

        return None

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

    def compare_expr(self, expr: Expression):
        assert expr.operator in COMPARISONS
        (left, right) = expr.args
        left_path = dot_path(left)
        right_path = dot_path(right)

        # Normalize partial to LHS.
        if right_path and not left_path:
            expr = reflect_expr(expr)
            left, right = right, left
            left_path, right_path = right_path, left_path

        left_field = self.translate_path_to_field(left_path)

        if left_path and right_path:
            # partial cmp partial
            # move `self` to a subdependency of `right`
            right_var = right_path[0]
            self.move_to_subquery(right_var)
            left_field = self.translate_path_to_field(left_path)
            right_field = self.translate_path_to_field(right_path)
            self.filter &= COMPARISONS["Unify"](left_field, F(right_field))
        else:
            # partial cmp grounded
            assert left_path
            if isinstance(right, Model):
                right = right.pk
            self.filter &= COMPARISONS[expr.operator](left_field, right)

    def in_expr(self, expr: Expression):
        assert expr.operator == "In"
        (left, right) = expr.args
        right_path = dot_path(right)

        if left == self.name and right_path:
            if right_path[1:]:
                # _this in _this.foo.bar
                # _this in _some_var.foo.bar
                right_var = right_path[0]
                # move `_this` to a subdependency of `right`
                self.move_to_subquery(right_var)
                left_field = "__".join(self.path)
                right_field = self.translate_path_to_field(right_path)
                self.filter &= COMPARISONS["Unify"](left_field, F(right_field))
            else:
                # _this in _this
                # _this in _some_var
                raise UnsupportedError(f"Unsupported partial expression: {expr}")
        elif isinstance(left, Variable) and right_path:
            if right_path[1:]:
                # var in _this.foo.bar
                # var in other_var.foo.bar

                # Left is a variable => apply constraints to the subquery.
                # (and left isn't _this_ variable)
                root = self.root()
                query = root.get_query_from_var(left)
                if query:
                    # LHS already exists, so lets create a new constraint
                    left_field = self.translate_path_to_field((left, "pk"))
                    right_field = self.translate_path_to_field(right_path)
                    self.filter &= COMPARISONS["Unify"](left_field, F(right_field))
                else:
                    # left does not exist _anywhere_ so lets create the subquery
                    # here
                    model = get_model_by_path(self.model, right_path[1:])
                    self.variables[left] = FilterBuilder(
                        model, parent=self, name=left, path=self.path + right_path[1:]
                    )
            else:
                # var in _this
                # var in other_var
                raise UnsupportedError(f"Unsupported partial expression: {expr}")
        else:
            # <value> in <partial>
            self.filter &= COMPARISONS["Unify"](
                self.translate_path_to_field(right_path), left
            )

    def not_expr(self, expr: Expression):
        assert expr.operator == "Not"
        assert expr.args[0].operator == "Isa"
        fb = FilterBuilder(self.model, parent=self.parent)
        fb.translate_expr(expr.args[0])
        self.filter &= ~fb.finish()

    def finish(self):
        """For every subquery, construct a filter to make sure the result set is non-empty"""
        subq_filter = Q()
        for subq in self.variables.values():
            subq_filter |= subq.finish()
        self.filter &= subq_filter
        if len(self.filter) == 0:
            self.filter &= COMPARISONS["Eq"](
                self.translate_path_to_field((self.name, "isnull")), False
            )
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
