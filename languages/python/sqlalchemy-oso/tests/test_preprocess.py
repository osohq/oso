import pytest

from collections import defaultdict

from polar.expression import Expression, Pattern
from polar.variable import Variable

from sqlalchemy_oso.preprocess import preprocess, preprocess_expression, find_related_variables


@pytest.mark.parametrize("test_expression, related", [
    (Expression("Unify", [Variable("_this"), 1]), {}),
    (Expression("And", [
        Expression("In", [Variable("_tag"), Expression("Dot", [Variable("_this"), "tags"])])
    ]), {
        Variable("_this"): {Variable("_tag")}
    }),
    (Expression("And", [
        Expression("In", [Variable("_tag"), Expression("Dot", [Variable("_this"), "tags"])]),
        Expression("In", [Variable("_user"), Expression("Dot", [Variable("_this"), "users"])])
    ]), {
        Variable("_this"): {Variable("_tag"), Variable("_user")}
    }),
    (Expression("And", [
        Expression("In", [Variable("_tag"), Expression("Dot", [Variable("_this"), "tags"])]),
        Expression("In", [Variable("_user"), Expression("Dot", [Variable("_this"), "users"])]),
        Expression("In", [Variable("_post"), Expression("Dot", [Variable("_tag"), "posts"])])
    ]), {
        Variable("_this"): {Variable("_tag"), Variable("_user")},
        Variable("_tag"): {Variable("_post")},
        Variable("_post"): {Variable("_tag")},
    }),
])
def test_find_related_variables(test_expression, related):
    assert find_related_variables(test_expression) == related


def test_preprocess_nested_many_many():
    _this = Variable("_this")
    expression = Expression("And", [
        Expression("Isa", [_this, Pattern("Post", {})]),
        Expression("In", [Variable("_tag_16"), Expression("Dot", [_this, "tags"])]),
        Expression("In", [Variable("_user_18"), Expression("Dot", [Variable("_tag_16"), "users"])]),
        Expression("Unify", ["admin", Expression("Dot", [Variable("_user_18"), "username"])])
    ])

    vars = defaultdict(list)
    new_expression = preprocess_expression(expression, vars)

    assert new_expression == Expression("And", [
        Expression("Isa", [_this, Pattern("Post", {})]),
        Expression("In", [Variable("_tag_16"), Expression("Dot", [_this, "tags"])]),
    ])

    assert vars == {
        Variable("_tag_16"): [
            Expression("In", [Variable("_user_18"), Expression("Dot",
                                                               [Variable("_tag_16"),
                                                                "users"])])],
        Variable("_user_18"): [Expression("Unify", ["admin", Expression("Dot", [Variable("_user_18"),
                                                                                "username"])])]
    }

    assert preprocess(expression) == Expression("And", [
        Expression("Isa", [_this, Pattern("Post", {})]),
        Expression("In", [Variable("_tag_16"), Expression("Dot", [_this, "tags"])]),
    ])

