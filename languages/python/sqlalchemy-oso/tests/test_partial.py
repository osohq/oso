"""Unit tests for partial implementation."""
from polar.expression import Expression
from polar.variable import Variable

from sqlalchemy_oso.partial import dot_op_path


def test_dot_op_path():
    single = Expression("Dot", [Variable("_this"), "created_by"])
    assert dot_op_path(single) == ["created_by"]

    double = Expression("Dot", [single, "username"])
    assert dot_op_path(double) == ["created_by", "username"]

    triple = Expression("Dot", [double, "first"])
    assert dot_op_path(triple) == ["created_by", "username", "first"]
