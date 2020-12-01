from polar import Variable, Expression
from polar.partial import dot_path


def test_dot_path():
    single = Expression("Dot", [Variable("_this"), "created_by"])
    assert dot_path(single) == ("created_by",)

    double = Expression("Dot", [single, "username"])
    assert dot_path(double) == ("created_by", "username")

    triple = Expression("Dot", [double, "first"])
    assert dot_path(triple) == ("created_by", "username", "first")
