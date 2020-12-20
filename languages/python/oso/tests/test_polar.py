from datetime import datetime
from math import inf, isnan, nan
from pathlib import Path

from polar import (
    polar_class,
    exceptions,
    Polar,
    Predicate,
    Variable,
    Partial,
    Expression,
    Pattern,
)
from polar.partial import TypeConstraint
from polar.exceptions import InvalidCallError

import pytest


def test_anything_works(polar, query):
    polar.load_str("f(1);")
    results = query("f(x)")
    assert results[0]["x"] == 1

    results = query("f(y)")
    assert results[0]["y"] == 1


def test_helpers(polar, load_file, query, qeval, qvar):
    load_file(Path(__file__).parent / "test_file.polar")  # f(1);
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]


def test_load_function(polar, query, qvar):
    """Make sure the load function works."""
    # Loading the same file twice doesn't mess stuff up.
    filename = Path(__file__).parent / "test_file.polar"
    polar.load_file(filename)
    with pytest.raises(exceptions.PolarRuntimeError) as e:
        polar.load_file(filename)
    assert (
        str(e.value)
        == f"Problem loading file: File {filename} has already been loaded."
    )

    renamed = Path(__file__).parent / "test_file_renamed.polar"
    with pytest.raises(exceptions.PolarRuntimeError) as e:
        polar.load_file(renamed)

    expected = f"Problem loading file: A file with the same contents as {renamed} named {filename} has already been loaded."
    assert str(e.value) == expected
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]

    polar.clear_rules()
    polar.load_file(Path(__file__).parent / "test_file.polar")
    polar.load_file(Path(__file__).parent / "test_file_gx.polar")
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert query("g(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]


def test_clear_rules(polar, query):
    class Test:
        pass

    polar.register_class(Test)
    polar.load_str("f(x) if x = 1;")
    assert len(query("f(1)")) == 1
    assert len(query("x = new Test()")) == 1
    polar.clear_rules()
    assert len(query("f(1)")) == 0
    assert len(query("x = new Test()")) == 1


def test_load_and_query():
    p = Polar()
    p.load_str("f(1); f(2); ?= f(1); ?= not f(3);")

    with pytest.raises(exceptions.OsoError):
        p.load_str("g(1); ?= g(2);")


def test_predicate(polar, qvar, query):
    """Test that predicates can be converted to and from python."""
    polar.load_str("f(x) if x = pred(1, 2);")
    assert qvar("f(x)", "x") == [Predicate("pred", [1, 2])]

    assert query(Predicate(name="f", args=[Predicate("pred", [1, 2])])) == [{}]


def test_query(load_file, polar, query):
    """Test that queries work with variable arguments"""

    load_file(Path(__file__).parent / "test_file.polar")
    # plaintext polar query: query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]

    assert query(Predicate(name="f", args=[Variable("a")])) == [
        {"a": 1},
        {"a": 2},
        {"a": 3},
    ]


def test_instance_cache(polar, qeval, query):
    class Counter:
        count = 0

        def __init__(self):
            self.__class__.count += 1

    polar.register_class(Counter)
    polar.load_str("f(c: Counter) if c.count > 0;")

    assert Counter.count == 0
    c = Counter()
    assert Counter.count == 1
    assert query(Predicate(name="f", args=[c]))
    assert Counter.count == 1
    assert c not in polar.host.instances.values()


# TODO: this should be in integration tests
def test_in(polar, qeval):
    polar.load_str("g(x, y) if not x in y;")
    polar.load_str("f(x) if not (x=1 or x=2);")
    assert not qeval("f(1)")
    assert qeval("g(4, [1,2,3])")
    assert not qeval("g(1, [1,1,1])")


def test_datetime(polar, query):
    # test datetime comparison
    t1 = datetime(2020, 5, 25)
    t2 = datetime.now()
    t3 = datetime(2030, 5, 25)
    t4 = datetime(2020, 5, 26)

    polar.load_str("lt(a, b) if a < b;")
    assert query(Predicate("lt", [t1, t2]))
    assert not query(Predicate("lt", [t2, t1]))

    # test creating datetime from polar
    polar.load_str("dt(x) if x = new Datetime(year: 2020, month: 5, day: 25);")
    assert query(Predicate("dt", [Variable("x")])) == [{"x": datetime(2020, 5, 25)}]
    polar.load_str("ltnow(x) if x < Datetime.now();")
    assert query(Predicate("ltnow", [t1]))
    assert not query(Predicate("ltnow", [t3]))

    polar.load_str(
        "timedelta(a: Datetime, b: Datetime) if a.__sub__(b) == new Timedelta(days: 1);"
    )
    assert query(Predicate("timedelta", [t4, t1]))


def test_other_constants(polar, qvar):
    """Test that other objects may be registered as constants."""
    d = {"a": 1}
    polar.register_constant(d, "d")
    assert qvar("x = d.a", "x") == [1]


def test_host_methods(qeval):
    assert qeval('x = "abc" and x.startswith("a") = true and x.find("bc") = 1')
    assert qeval("i = 4095 and i.bit_length() = 12")
    assert qeval('f = 3.14159 and f.hex() = "0x1.921f9f01b866ep+1"')
    assert qeval("l = [1, 2, 3] and l.index(3) = 2 and l.copy() = [1, 2, 3]")
    assert qeval('d = {a: 1} and d.get("a") = 1 and d.get("b", 2) = 2')


## TODO: should these be on integration side?
def test_inf_nan(polar, qeval, query):
    polar.register_constant(inf, "inf")
    polar.register_constant(-inf, "neg_inf")
    polar.register_constant(nan, "nan")

    assert isnan(query("x = nan")[0]["x"])
    assert not query("nan = nan")

    assert query("x = inf")[0]["x"] == inf
    assert qeval("inf = inf")

    assert query("x = neg_inf")[0]["x"] == -inf
    assert qeval("neg_inf = neg_inf")

    assert not query("inf = neg_inf")
    assert not query("inf < neg_inf")
    assert qeval("neg_inf < inf")


def test_register_constants_with_decorator():
    @polar_class
    class RegisterDecoratorTest:
        x = 1

    p = Polar()
    p.load_str("foo_rule(x: RegisterDecoratorTest, y) if y = 1;")
    p.load_str("foo_class_attr(y) if y = RegisterDecoratorTest.x;")
    assert (
        next(p.query_rule("foo_rule", RegisterDecoratorTest(), Variable("y")))[
            "bindings"
        ]["y"]
        == 1
    )
    assert next(p.query_rule("foo_class_attr", Variable("y")))["bindings"]["y"] == 1

    p = Polar()
    p.load_str("foo_rule(x: RegisterDecoratorTest, y) if y = 1;")
    p.load_str("foo_class_attr(y) if y = RegisterDecoratorTest.x;")
    assert (
        next(p.query_rule("foo_rule", RegisterDecoratorTest(), Variable("y")))[
            "bindings"
        ]["y"]
        == 1
    )
    assert next(p.query_rule("foo_class_attr", Variable("y")))["bindings"]["y"] == 1


def test_static_method(polar, qeval):
    class Foo(list):
        @staticmethod
        def plus_one(x):
            return x + 1

        def map(self, f):
            return [f(x) for x in self]

    polar.register_class(Foo)
    polar.load_str("f(x: Foo) if x.map(Foo.plus_one) = [2, 3, 4];")
    assert next(polar.query_rule("f", Foo([1, 2, 3])))


def unwrap_and(x):
    assert isinstance(x, Expression)
    assert x.operator == "And"
    if len(x.args) == 1:
        return x.args[0]
    else:
        return x.args


def test_partial(polar):
    polar.load_str("f(1);")
    polar.load_str("f(x) if x = 1 and x = 2;")

    results = polar.query_rule("f", Partial("x"))
    first = next(results)

    x = first["bindings"]["x"]
    assert Expression("Unify", [Variable("_this"), 1])

    second = next(results)
    x = second["bindings"]["x"]

    # Top level should be and
    and_args = unwrap_and(x)
    assert and_args[0] == Expression("Unify", [Variable("_this"), 1])
    assert and_args[1] == Expression("Unify", [Variable("_this"), 2])

    polar.load_str("g(x) if x.bar = 1 and x.baz = 2;")

    results = polar.query_rule("g", Partial("x"))
    first = next(results)

    x = first["bindings"]["x"]
    and_args = unwrap_and(x)
    assert len(and_args) == 2
    assert and_args[0] == Expression(
        "Unify", [Expression("Dot", [Variable("_this"), "bar"]), 1]
    )
    assert and_args[1] == Expression(
        "Unify", [Expression("Dot", [Variable("_this"), "baz"]), 2]
    )


def test_partial_constraint(polar):
    class User:
        pass

    class Post:
        pass

    polar.register_class(User)
    polar.register_class(Post)

    polar.load_str("f(x: User) if x.user = 1;")
    polar.load_str("f(x: Post) if x.post = 1;")

    partial = Partial("x", TypeConstraint("User"))
    results = polar.query_rule("f", partial)

    first = next(results)["bindings"]["x"]
    and_args = unwrap_and(first)

    assert len(and_args) == 2

    assert and_args[0] == Expression("Isa", [Variable("_this"), Pattern("User", {})])

    unify = and_args[1]
    assert unify == Expression(
        "Unify", [Expression("Dot", [Variable("_this"), "user"]), 1]
    )

    with pytest.raises(StopIteration):
        next(results)

