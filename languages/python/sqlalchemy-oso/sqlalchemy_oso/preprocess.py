"""Convert expressions from oso into a format that the SQLAlchemy translation can use."""

from collections import defaultdict
from typing import Dict, Optional, List, Iterable, Set

from polar.expression import Expression
from polar.variable import Variable
from polar.exceptions import UnsupportedError


TGroupedExpressions = Dict[Variable, List[Expression]]


def preprocess(expression: Expression) -> Expression:
    # Collect expressions that constrain variables besides _this.
    variables: TGroupedExpressions = defaultdict(list)
    new_expr = preprocess_expression(expression, variables)
    assert new_expr is not None

    # Join each expression by AND.
    expressions = {var: Expression("And", args) for var, args in variables.items()}

    # Subsitute _this for each variable.
    expressions = {var: sub_this(var, expression) for var, expression in expressions.items()}

    # Subsitute new expressions for variables in original expression.
    for var, expr in expressions.items():
        new_expr = sub_var(var, expr, preprocess(new_expr))

    return new_expr


def preprocess_expression(expression: Expression, variables: TGroupedExpressions) -> Optional[Expression]:
    """Collect expressions over variables in ``extract`` into ``variables``.

    Return the expression with those removed."""
    # Walk expression and collect variable expressions
    new_expr = expression
    if expression.operator == "And":
        new_expr = preprocess_and(expression, variables)
    elif expression.operator in ("Or", "Not"):
        raise UnsupportedError(f"{expression.operator}")
    else:
        new_expr = preprocess_leaf(expression, variables)

    return new_expr


def preprocess_and(expression: Expression, variables: TGroupedExpressions) -> Expression:
    new_expression = []

    for expression in expression.args:
        maybe_expr = preprocess_expression(expression, variables)
        if maybe_expr:
            new_expression.append(maybe_expr)

    return Expression("And", new_expression)


def get_variable(expression_or_variable):
    """Get variable out of nested dot or single variable."""
    if isinstance(expression_or_variable, Variable):
        return expression_or_variable
    elif isinstance(expression_or_variable, Expression):
        if expression_or_variable.operator == "Dot":
            return get_variable(expression_or_variable.args[0])

    return None


def is_this(variable):
    """Return true if ``variable`` is ``_this``."""
    return variable == Variable("_this")


def sub_this(variable: Variable, expression: Expression) -> Expression:
    """Substitute _this for ``variable`` in ``expression``."""
    return sub_var(variable, Variable("_this"), expression)


def sub_var(variable: Variable, value, expression: Expression) -> Expression:
    """Substitute ``value`` for ``variable`` in ``expression``."""
    new_expr = []
    for arg in expression.args:
        if isinstance(arg, Expression):
            arg = sub_var(variable, value, arg)
        elif arg == variable:
            arg = value

        new_expr.append(arg)

    return Expression(expression.operator, new_expr)


def preprocess_leaf(expression: Expression, variables: TGroupedExpressions) -> Optional[Expression]:
    """If leaf is a variable other than _this, add the expression to variables and return None."""
    assert len(expression.args) == 2
    left, right = expression.args
    left_var = get_variable(left)
    right_var = get_variable(right)

    if is_this(left_var) or is_this(right_var):
        return expression

    # We only extract by the right variable first to handle In properly.
    if right_var is not None:
        variables[right_var].append(expression)
        return None
    if left_var is not None:
        variables[left_var].append(expression)
        return None

    return expression


# NOTE: Didn't need to use find_related_variables, but I think there may be
# a more principled implementation that requires this.

def find_related_variables(expression: Expression) -> Dict[Variable, Iterable[Variable]]:
    """From an input expression, output variables that are compared to other variables.

    _this is special, and will only appear as a key in the output.
    other variables that are not _this will appear bi-directionally.

    Example:

        _tag_1 in _this.tags

        {
            "_this": ["_tag_1"]
        }

        _tag_1 in _this.tags and
        _user_1 in _this.users

        {
            "_this": ["_tag_1", "user_1"]
        }

        _tag_1 in _this.tags and
        _user_1 in _this.users and
        _post_1 in _tag_1 and
        _post_1.id = 1

        {
            "_this": ["_tag_1", "user_1"]
            "_tag_1": ["_post_1"],
            "post_1": ["_tag_1"]
        }
    """
    return find_related_variables_expression(expression)


def find_related_variables_expression(expression: Expression) -> Dict[Variable, Iterable[Variable]]:
    if expression.operator == "And":
        related_variables: Dict[Variable, Set[Variable]] = defaultdict(set)
        for expression in expression.args:
            rv = find_related_variables_expression(expression)
            for k, vs in rv.items():
                for var in vs:
                    related_variables[k].add(var)
        return related_variables
    elif expression.operator in ("Or", "Not"):
        raise UnsupportedError("Not implemented expression")
    else:
        return find_related_variables_leaf(expression)


def find_related_variables_leaf(expression: Expression) -> Dict[Variable, Iterable[Variable]]:
    assert len(expression.args) == 2
    left, right = expression.args
    left_var = get_variable(left)
    right_var = get_variable(right)
    left_var = get_variable(left)

    if left_var is None or right_var is None:
        # Both are not variables
        return {}

    if is_this(left_var):
        assert not is_this(right_var)
        return {left_var: [right_var]}
    elif is_this(right_var):
        assert not is_this(left_var)
        return {right_var: [left_var]}
    else:
        return {left_var: [right_var], right_var: [left_var]}
