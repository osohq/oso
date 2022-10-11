from polar import Expression, Variable
from polar.partial import dot_path


def test_dot_path():
    non_dot = Expression("And", [])
    assert dot_path(non_dot) == ()

    this = Variable("_this")
    assert dot_path(this) == (this,)

    var = Variable("x")
    assert dot_path(var) == (var,)

    single_dot = Expression("Dot", [this, "created_by"])
    assert dot_path(single_dot) == (this, "created_by")

    double_dot = Expression("Dot", [single_dot, "username"])
    assert dot_path(double_dot) == (this, "created_by", "username")

    triple_dot = Expression("Dot", [double_dot, "first"])
    assert dot_path(triple_dot) == (this, "created_by", "username", "first")
