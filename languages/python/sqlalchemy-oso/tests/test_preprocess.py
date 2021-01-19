from collections import defaultdict

from polar.expression import Expression, Pattern
from polar.variable import Variable

from sqlalchemy_oso.preprocess import preprocess, preprocess_expression


def test_preprocess_nested_many_many():
    _this = Variable("_this")
    expression = Expression(
        "And",
        [
            Expression("Isa", [_this, Pattern("Post", {})]),
            Expression("In", [Variable("_tag_16"), Expression("Dot", [_this, "tags"])]),
            Expression(
                "In",
                [
                    Variable("_user_18"),
                    Expression("Dot", [Variable("_tag_16"), "users"]),
                ],
            ),
            Expression(
                "Unify",
                ["admin", Expression("Dot", [Variable("_user_18"), "username"])],
            ),
        ],
    )

    vars = defaultdict(list)
    new_expression = preprocess_expression(expression, vars)

    assert new_expression == Expression(
        "And",
        [
            Expression("Isa", [_this, Pattern("Post", {})]),
            Expression("In", [Variable("_tag_16"), Expression("Dot", [_this, "tags"])]),
        ],
    )

    assert vars == {
        Variable("_tag_16"): [
            Expression(
                "In",
                [
                    Variable("_user_18"),
                    Expression("Dot", [Variable("_tag_16"), "users"]),
                ],
            )
        ],
        Variable("_user_18"): [
            Expression(
                "Unify",
                ["admin", Expression("Dot", [Variable("_user_18"), "username"])],
            )
        ],
    }

    users_expr = Expression(
        "And", [Expression("Unify", ["admin", Expression("Dot", [_this, "username"])])]
    )
    tags_expr = Expression(
        "And", [Expression("In", [users_expr, Expression("Dot", [_this, "users"])])]
    )

    assert preprocess(expression) == Expression(
        "And",
        [
            Expression("Isa", [_this, Pattern("Post", {})]),
            Expression("In", [tags_expr, Expression("Dot", [_this, "tags"])]),
        ],
    )
