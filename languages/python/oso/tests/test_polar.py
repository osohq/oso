from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from math import inf, isnan, nan
from pathlib import Path
from typing import List

import pytest

from polar import Expression, Pattern, Polar, Predicate, Variable, exceptions
from polar.errors import ValidationError
from polar.partial import TypeConstraint


def test_anything_works(polar, query):
    polar.load_str("f(1);")
    results = query("f(x)")
    assert results[0]["x"] == 1

    results = query("f(y)")
    assert results[0]["y"] == 1


def test_helpers(polar, query, qvar):
    polar.load_file(Path(__file__).parent / "test_file.polar")  # f(1);
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]


def test_data_conversions(polar, qvar):
    polar.load_str('a(1);b("two");c(true);d([1,"two",true]);')
    assert qvar("a(x)", "x", one=True) == 1
    assert qvar("b(x)", "x", one=True) == "two"
    assert qvar("c(x)", "x", one=True)
    assert qvar("d(x)", "x", one=True) == [1, "two", True]
    x = qvar("x = y", "x", one=True)
    assert str(x) == "Variable('y')"
    assert repr(x) == "Variable('y')"


def test_load_function(polar, query, qvar):
    """Make sure the load function works."""
    filename = Path(__file__).parent / "test_file.polar"
    with pytest.raises(exceptions.ValidationError) as e:
        polar.load_files([filename, filename])
    assert str(e.value).startswith(
        f"Problem loading file: File {filename} has already been loaded."
    )

    renamed = Path(__file__).parent / "test_file_renamed.polar"
    with pytest.raises(exceptions.ValidationError) as e:
        polar.load_files([filename, renamed])
    expected = f"Problem loading file: A file with the same contents as {renamed} named {filename} has already been loaded."
    assert str(e.value).startswith(expected)

    polar.load_file(filename)
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]

    polar.clear_rules()
    polar.load_files(
        [
            Path(__file__).parent / "test_file.polar",
            Path(__file__).parent / "test_file_gx.polar",
        ]
    )
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert query("g(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]


def test_load_multiple_files_same_name_different_path(polar, qvar):
    file1 = Path(__file__).parent / "test_file.polar"
    file2 = Path(__file__).parent / "other/test_file.polar"
    polar.load_files([file1, file2])
    assert qvar("f(x)", "x") == [1, 2, 3]
    assert qvar("g(x)", "x") == [1, 2, 3]


def test_clear_rules(polar, query):
    class Test:
        pass

    polar.register_class(Test)
    polar.load_str("f(x) if x = 1;")
    assert len(query("f(1)")) == 1
    assert len(query("x = new Test()")) == 1

    polar.clear_rules()

    with pytest.raises(exceptions.PolarRuntimeError) as e:
        query("f(1)") == []
    assert "Query for undefined rule `f`" in str(e.value)
    assert len(query("x = new Test()")) == 1


def test_external(polar, qvar, qeval):
    class Bar:
        def y(self):
            return "y"

    class Foo:
        def __init__(self, a="a"):
            self.a = a

        def b(self):
            return "b"

        @classmethod
        def c(cls):
            assert issubclass(cls, Foo)
            return "c"

        def d(self, x):
            return x

        def bar(self):
            return Bar()

        def e(self):
            return [1, 2, 3]

        def f(self):
            yield [1, 2, 3]
            yield [4, 5, 6]
            yield 7

        def g(self):
            return {"hello": "world"}

        def h(self):
            return True

    polar.register_class(Foo)
    assert qvar("new Foo().a = x", "x", one=True) == "a"
    with pytest.raises(
        exceptions.InvalidCallError, match="tried to call 'a' but it is not callable"
    ):
        assert not qeval("new Foo().a() = x")
    assert not qvar("new Foo().b = x", "x", one=True) == "b"
    assert qvar("new Foo().b() = x", "x", one=True) == "b"
    assert not qvar("Foo.c = x", "x", one=True) == "c"
    assert qvar("Foo.c() = x", "x", one=True) == "c"
    assert qvar("new Foo() = f and f.a = x", "x", one=True) == "a"
    assert qvar("new Foo().bar().y() = x", "x", one=True) == "y"
    assert qvar("new Foo().e() = x", "x", one=True) == [1, 2, 3]
    assert qvar("x in new Foo().e()", "x") == [1, 2, 3]
    assert qvar("x in new Foo().f()", "x") == [[1, 2, 3], [4, 5, 6], 7]
    assert qvar("new Foo().g().hello = x", "x", one=True) == "world"
    assert qvar("new Foo().h() = x", "x", one=True) is True


def test_class_specializers(polar, qvar, query):
    class A:
        def a(self):
            return "A"

        def x(self):
            return "A"

    class B(A):
        def b(self):
            return "B"

        def x(self):
            return "B"

    class C(B):
        def c(self):
            return "C"

        def x(self):
            return "C"

    class X:
        def x(self):
            return "X"

    polar.register_class(A)
    polar.register_class(B)
    polar.register_class(C)
    polar.register_class(X)

    rules = """
    test(_: A);
    test(_: B);

    try(_: B, res) if res = 2;
    try(_: C, res) if res = 3;
    try(_: A, res) if res = 1;
    """
    polar.load_str(rules)

    assert qvar("new A().a() = x", "x", one=True) == "A"
    assert qvar("new A().x() = x", "x", one=True) == "A"
    assert qvar("new B().a() = x", "x", one=True) == "A"
    assert qvar("new B().b() = x", "x", one=True) == "B"
    assert qvar("new B().x() = x", "x", one=True) == "B"
    assert qvar("new C().a() = x", "x", one=True) == "A"
    assert qvar("new C().b() = x", "x", one=True) == "B"
    assert qvar("new C().c() = x", "x", one=True) == "C"
    assert qvar("new C().x() = x", "x", one=True) == "C"
    assert qvar("new X().x() = x", "x", one=True) == "X"

    assert len(query("test(new A())")) == 1
    assert len(query("test(new B())")) == 2

    assert qvar("try(new A(), x)", "x") == [1]
    assert qvar("try(new B(), x)", "x") == [2, 1]
    assert qvar("try(new C(), x)", "x") == [3, 2, 1]
    assert qvar("try(new X(), x)", "x") == []


def test_dict_specializers(polar, qvar, query):
    class Animal:
        def __init__(self, species=None, genus=None, family=None):
            self.genus = genus
            self.species = species
            self.family = family

    polar.register_class(Animal)

    rules = """
    what_is(_: {genus: "canis"}, res) if res = "canine";
    what_is(_: {species: "canis lupus", genus: "canis"}, res) if res = "wolf";
    what_is(_: {species: "canis familiaris", genus: "canis"}, res) if res = "dog";
    """
    polar.load_str(rules)

    wolf = 'new Animal(species: "canis lupus", genus: "canis", family: "canidae")'
    dog = 'new Animal(species: "canis familiaris", genus: "canis", family: "canidae")'
    canine = 'new Animal(genus: "canis", family: "canidae")'

    assert len(query(f"what_is({wolf}, res)")) == 2
    assert len(query(f"what_is({dog}, res)")) == 2
    assert len(query(f"what_is({canine}, res)")) == 1

    assert qvar(f"what_is({wolf}, res)", "res") == ["wolf", "canine"]
    assert qvar(f"what_is({dog}, res)", "res") == ["dog", "canine"]
    assert qvar(f"what_is({canine}, res)", "res") == ["canine"]


def test_class_field_specializers(polar, qvar, query):
    class Animal:
        def __init__(self, species=None, genus=None, family=None):
            self.genus = genus
            self.species = species
            self.family = family

    polar.register_class(Animal)

    rules = """
    what_is(_: Animal, res) if res = "animal";
    what_is(_: Animal{genus: "canis"}, res) if res = "canine";
    what_is(_: Animal{family: "canidae"}, res) if res = "canid";
    what_is(_: Animal{species: "canis lupus", genus: "canis"}, res) if res = "wolf";
    what_is(_: Animal{species: "canis familiaris", genus: "canis"}, res) if res = "dog";
    what_is(_: Animal{species: s, genus: "canis"}, res) if res = s;
    """
    polar.load_str(rules)

    wolf = 'new Animal(species: "canis lupus", genus: "canis", family: "canidae")'
    dog = 'new Animal(species: "canis familiaris", genus: "canis", family: "canidae")'
    canine = 'new Animal(genus: "canis", family: "canidae")'
    canid = 'new Animal(family: "canidae")'
    animal = "new Animal()"

    assert len(query(f"what_is({wolf}, res)")) == 5
    assert len(query(f"what_is({dog}, res)")) == 5
    assert len(query(f"what_is({canine}, res)")) == 4
    assert len(query(f"what_is({canid}, res)")) == 2
    assert len(query(f"what_is({animal}, res)")) == 1

    assert qvar(f"what_is({wolf}, res)", "res") == [
        "wolf",
        "canis lupus",
        "canine",
        "canid",
        "animal",
    ]
    assert qvar(f"what_is({dog}, res)", "res") == [
        "dog",
        "canis familiaris",
        "canine",
        "canid",
        "animal",
    ]
    assert qvar(f"what_is({canine}, res)", "res") == [None, "canine", "canid", "animal"]
    assert qvar(f"what_is({canid}, res)", "res") == ["canid", "animal"]
    assert qvar(f"what_is({animal}, res)", "res") == ["animal"]


def test_specializers_mixed(polar, qvar, query):
    class Animal:
        def __init__(self, species=None, genus=None, family=None):
            self.genus = genus
            self.species = species
            self.family = family

    polar.register_class(Animal)

    # load rules
    rules = """
    what_is(_: Animal, res) if res = "animal_class";
    what_is(_: Animal{genus: "canis"}, res) if res = "canine_class";
    what_is(_: {genus: "canis"}, res) if res = "canine_dict";
    what_is(_: Animal{family: "canidae"}, res) if res = "canid_class";
    what_is(_: {species: "canis lupus", genus: "canis"}, res) if res = "wolf_dict";
    what_is(_: {species: "canis familiaris", genus: "canis"}, res) if res = "dog_dict";
    what_is(_: Animal{species: "canis lupus", genus: "canis"}, res) if res = "wolf_class";
    what_is(_: Animal{species: "canis familiaris", genus: "canis"}, res) if res = "dog_class";
    """
    polar.load_str(rules)

    wolf = 'new Animal(species: "canis lupus", genus: "canis", family: "canidae")'
    dog = 'new Animal(species: "canis familiaris", genus: "canis", family: "canidae")'
    canine = 'new Animal(genus: "canis", family: "canidae")'

    wolf_dict = '{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog_dict = '{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine_dict = '{genus: "canis", family: "canidae"}'

    # test number of results
    assert len(query(f"what_is({wolf}, res)")) == 6
    assert len(query(f"what_is({dog}, res)")) == 6
    assert len(query(f"what_is({canine}, res)")) == 4
    assert len(query(f"what_is({wolf_dict}, res)")) == 2
    assert len(query(f"what_is({dog_dict}, res)")) == 2
    assert len(query(f"what_is({canine_dict}, res)")) == 1

    # test rule ordering for instances
    assert qvar(f"what_is({wolf}, res)", "res") == [
        "wolf_class",
        "canine_class",
        "canid_class",
        "animal_class",
        "wolf_dict",
        "canine_dict",
    ]
    assert qvar(f"what_is({dog}, res)", "res") == [
        "dog_class",
        "canine_class",
        "canid_class",
        "animal_class",
        "dog_dict",
        "canine_dict",
    ]
    assert qvar(f"what_is({canine}, res)", "res") == [
        "canine_class",
        "canid_class",
        "animal_class",
        "canine_dict",
    ]

    # test rule ordering for dicts
    assert qvar(f"what_is({wolf_dict}, res)", "res") == ["wolf_dict", "canine_dict"]
    assert qvar(f"what_is({dog_dict}, res)", "res") == ["dog_dict", "canine_dict"]
    assert qvar(f"what_is({canine_dict}, res)", "res") == ["canine_dict"]


def test_load_and_query():
    p = Polar()
    p.load_str("f(1); f(2); ?= f(1); ?= not f(3);")

    with pytest.raises(exceptions.OsoError):
        p.load_str("g(1); ?= g(2);")


def test_parser_errors(polar):
    # IntegerOverflow
    rules = """
    f(a) if a = 18446744073709551616;
    """
    with pytest.raises(exceptions.IntegerOverflow) as e:
        polar.load_str(rules)
    assert str(e.value).startswith(
        "'18446744073709551616' caused an integer overflow at line 2, column 17"
    )

    # InvalidTokenCharacter
    rules = """
    f(a) if a = "this is not
    allowed";
    """
    with pytest.raises(exceptions.InvalidTokenCharacter) as e:
        polar.load_str(rules)
    assert str(e.value).startswith(
        "'\\n' is not a valid character. Found in this is not at line 2, column 29"
    )

    # TODO(gj): figure out what changed.
    rules = """
    f(a) if a = "this is not allowed\0"""

    with pytest.raises(exceptions.InvalidTokenCharacter) as e:
        polar.load_str(rules)
    assert str(e.value).startswith(
        "'\\0' is not a valid character. Found in this is not allowed\\0 at line 2, column 17"
    ), e

    # InvalidToken -- not sure what causes this

    # UnrecognizedEOF
    rules = """
    f(a)
    """
    with pytest.raises(exceptions.UnrecognizedEOF) as e:
        polar.load_str(rules)
    assert str(e.value).startswith(
        "hit the end of the file unexpectedly. Did you forget a semi-colon at line 2, column 9"
    )

    # UnrecognizedToken
    rules = """
    1;
    """
    with pytest.raises(exceptions.UnrecognizedToken) as e:
        polar.load_str(rules)
    assert str(e.value).startswith(
        "did not expect to find the token '1' at line 2, column 5"
    )

    # ExtraToken -- not sure what causes this


def test_runtime_errors(polar, query):
    rules = """
    foo(a,b) if a in b;
    """
    polar.load_str(rules)
    with pytest.raises(exceptions.PolarRuntimeError) as e:
        query("foo(1,2)")
    assert """trace (most recent evaluation last):
  002: foo(1,2)
    in query at line 1, column 1
  001: a in b
    in rule foo at line 2, column 17

Type error: can only use `in` on an iterable value, this is Number(Integer(2)) at line 1, column 7""" in str(
        e.value
    )


def test_lookup_errors(polar, query):
    class Foo:
        def foo(self):
            return "foo"

    polar.register_class(Foo)

    # Unify with an invalid field doesn't error.
    assert query('new Foo() = {bar: "bar"}') == []
    # Dot op with an invalid field does error.
    with pytest.raises(exceptions.PolarRuntimeError) as e:
        query('new Foo().bar = "bar"') == []
    assert "Application error: 'Foo' object has no attribute 'bar'" in str(e.value)


def test_return_list(polar, query):
    class User:
        def groups(self):
            return ["engineering", "social", "admin"]

    polar.register_class(User)

    # for testing lists
    polar.load_str('allow(actor: User, "join", "party") if "social" in actor.groups();')

    assert query(Predicate(name="allow", args=[User(), "join", "party"]))


def test_host_native_unify(query):
    """Test that unification works across host and native data"""
    assert query("new Integer(1) = 1")
    assert query('new String("foo") = "foo"')
    assert query("new List([1,2,3]) = [1,2,3]")


def test_query(polar, query):
    """Test that queries work with variable arguments"""

    polar.load_file(Path(__file__).parent / "test_file.polar")
    # plaintext polar query: query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]

    assert query(Predicate(name="f", args=[Variable("a")])) == [
        {"a": 1},
        {"a": 2},
        {"a": 3},
    ]


def test_constructor(polar, qvar):
    """Test that class constructor is called correctly with constructor syntax."""

    class Foo:
        def __init__(self, a, b, bar, baz):
            self.a = a
            self.b = b
            self.bar = bar
            self.baz = baz

    polar.register_class(Foo)

    # test positional args
    instance = qvar(
        "instance = new Foo(1,2,3,4)",
        "instance",
        one=True,
    )
    assert instance.a == 1
    assert instance.b == 2
    assert instance.bar == 3
    assert instance.baz == 4

    # test positional and kwargs
    instance = qvar(
        "instance = new Foo(1, 2, bar: 3, baz: 4)",
        "instance",
        one=True,
    )
    assert instance.a == 1
    assert instance.b == 2
    assert instance.bar == 3
    assert instance.baz == 4

    # test kwargs
    instance = qvar(
        "instance = new Foo(bar: 3, a: 1, baz: 4, b: 2)",
        "instance",
        one=True,
    )
    assert instance.a == 1
    assert instance.b == 2
    assert instance.bar == 3
    assert instance.baz == 4


def test_constructor_error(polar, query):
    """Test that external instance constructor errors cause a PolarRuntimeError"""

    class Foo:
        def __init__(self):
            raise RuntimeError("o no")

    polar.register_class(Foo)
    with pytest.raises(exceptions.PolarRuntimeError):
        query("x = new Foo()")


def test_instance_cache(polar, query):
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


def test_in(polar, qeval):
    polar.load_str(
        """g(x, y) if not x in y;
           f(x) if not (x=1 or x=2);"""
    )
    assert not qeval("f(1)")
    assert qeval("g(4, [1,2,3])")
    assert not qeval("g(1, [1,1,1])")


def test_unify(polar, qeval):
    class Foo:
        def __init__(self, foo):
            self.foo = foo

        def __eq__(self, other):
            if isinstance(other, Foo):
                return self.foo == other.foo
            return False

    polar.register_class(Foo)

    polar.load_str("foo() if new Foo(foo: 1) = new Foo(foo: 1);")
    assert qeval("foo()")


def test_external_op(polar, query):
    class A:
        def __init__(self, a):
            self.a = a

        def __gt__(self, other):
            return self.a > other.a

        def __lt__(self, other):
            return self.a < other.a

        def __eq__(self, other):
            return self.a == other.a

    polar.register_class(A)

    a1 = A(1)
    a2 = A(2)

    polar.load_str(
        """lt(a, b) if a < b;
           gt(a, b) if a > b;"""
    )
    assert query(Predicate("lt", [a1, a2]))
    assert not query(Predicate("lt", [a2, a1]))
    assert query(Predicate("gt", [a2, a1]))


def test_datetime(polar, query):
    # test datetime comparison
    t1 = datetime(2020, 5, 25)
    t2 = datetime.now()
    t3 = datetime(2030, 5, 25)
    t4 = datetime(2020, 5, 26)

    polar.load_str("lt(a, b) if a < b;")
    assert query(Predicate("lt", [t1, t2]))
    assert not query(Predicate("lt", [t2, t1]))

    polar.clear_rules()

    # test creating datetime from polar
    polar.load_str("dt(x) if x = new Datetime(year: 2020, month: 5, day: 25);")
    assert query(Predicate("dt", [Variable("x")])) == [{"x": datetime(2020, 5, 25)}]

    polar.clear_rules()

    polar.load_str("ltnow(x) if x < Datetime.now();")
    assert query(Predicate("ltnow", [t1]))
    assert not query(Predicate("ltnow", [t3]))

    polar.clear_rules()

    polar.load_str(
        "timedelta(a: Datetime, b: Datetime) if a.__sub__(b) == new Timedelta(days: 1);"
    )
    assert query(Predicate("timedelta", [t4, t1]))


def test_nil(polar, query, qvar):
    """Test that nil is pre-registered as None."""
    polar.load_str("null(nil);")
    assert qvar("null(x)", "x") == [None]
    assert query(Predicate("null", [None])) == [{}]
    assert not query(Predicate("null", [[]]))


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


def test_unbound_variable(polar, query):
    """Test that unbound variable is returned."""
    polar.load_str("rule(_, y) if y = 1;")

    first = query("rule(x, y)")[0]

    # y will be bound to 1
    first["y"] = 1

    # x should be unbound
    assert isinstance(first["x"], Variable)


def test_return_none(polar):
    class Foo:
        def this_is_none(self):
            return None

    polar.register_class(Foo)
    polar.load_str("f(x) if x.this_is_none() = nil;")
    assert len(list(polar.query_rule("f", Foo()))) == 1

    polar.clear_rules()

    polar.load_str("g(x) if x.this_is_none().bad_call() = 1;")
    with pytest.raises(exceptions.PolarRuntimeError) as e:
        list(polar.query_rule("g", Foo()))
    assert str(e.value).find(
        "Application error: 'NoneType' object has no attribute 'bad_call'"
    )


def test_static_method(polar):
    class Foo(list):
        @staticmethod
        def plus_one(x):
            return x + 1

        def map(self, f):
            return [f(x) for x in self]

    polar.register_class(Foo)
    polar.load_str("f(x: Foo) if x.map(Foo.plus_one) = [2, 3, 4];")
    assert next(polar.query_rule("f", Foo([1, 2, 3])))


def test_method_with_kwargs(polar, qvar):
    class Test:
        def kwarg_method(self, x=1, y=2):
            self.x = x
            self.y = y
            return True

    polar.register_class(Test)
    rules = """
    defaults(result) if
        test = new Test() and
        test.kwarg_method() and
        result = [test.x, test.y];

    kwargs(result) if
        test = new Test() and
        test.kwarg_method(y: 4, x: 3) and
        result = [test.x, test.y];

    args(result) if
        test = new Test() and
        test.kwarg_method(5, 6) and
        result = [test.x, test.y];

    mixed(result) if
        test = new Test() and
        test.kwarg_method(7, y: 8) and
        result = [test.x, test.y];
    """
    polar.load_str(rules)
    qvar("defaults(result)", "result") == [1, 2]
    qvar("kwargs(result)", "result") == [3, 4]
    qvar("args(result)", "result") == [5, 6]
    qvar("mixed(result)", "result") == [7, 8]


def unwrap_and(x):
    assert isinstance(x, Expression)
    assert x.operator == "And"
    if len(x.args) == 1:
        return x.args[0]
    else:
        return x.args


def test_partial_unification(polar):
    polar.load_str("f(x, y) if x = y;")
    x = Variable("x")
    y = Variable("y")
    results = polar.query_rule("f", x, y)
    result = next(results)["bindings"]
    assert result["x"] == y
    assert result["y"] == x
    with pytest.raises(StopIteration):
        next(results)


def test_partial(polar):
    polar.load_str(
        """f(1);
           f(x) if x = 1 and x = 2;"""
    )

    results = polar.query_rule("f", Variable("x"), accept_expression=True)
    first = next(results)

    x = first["bindings"]["x"]
    assert x == 1

    with pytest.raises(StopIteration):
        next(results)

    polar.clear_rules()

    polar.load_str("g(x) if x.bar = 1 and x.baz = 2;")

    results = polar.query_rule("g", Variable("x"), accept_expression=True)
    first = next(results)

    x = first["bindings"]["x"]
    and_args = unwrap_and(x)
    assert len(and_args) == 2
    assert and_args[0] == Expression(
        "Unify", [1, Expression("Dot", [Variable("_this"), "bar"])]
    )
    assert and_args[1] == Expression(
        "Unify", [2, Expression("Dot", [Variable("_this"), "baz"])]
    )


def test_partial_constraint(polar):
    class User:
        pass

    class Post:
        pass

    polar.register_class(User)
    polar.register_class(Post)

    polar.load_str(
        """f(x: User) if x.user = 1;
           f(x: Post) if x.post = 1;"""
    )

    x = Variable("x")
    results = polar.query_rule(
        "f", x, bindings={x: TypeConstraint(x, "User")}, accept_expression=True
    )

    first = next(results)["bindings"]["x"]
    and_args = unwrap_and(first)

    assert len(and_args) == 2

    assert and_args[0] == Expression("Isa", [Variable("_this"), Pattern("User", {})])

    unify = and_args[1]
    assert unify == Expression(
        "Unify", [1, Expression("Dot", [Variable("_this"), "user"])]
    )

    with pytest.raises(StopIteration):
        next(results)


def test_partial_rule_filtering(polar):
    class A:
        def __init__(self):
            self.c = C()

    class B:
        pass

    class C:
        pass

    class D:
        pass

    polar.register_class(A)
    polar.register_class(B)
    polar.register_class(C)
    polar.register_class(D)

    polar.load_str(
        """f(x: A) if g(x.c);
           g(_: B);
           g(_: C);
           g(_: D);"""
    )

    x = Variable("x")
    with pytest.raises(exceptions.PolarRuntimeError) as e:
        next(polar.query_rule("f", x, bindings={x: TypeConstraint(x, "A")}))
    assert str(e.value).startswith("No field c on A")


def test_iterators(polar, qeval, qvar):
    class Foo:
        pass

    polar.register_class(Foo)
    with pytest.raises(exceptions.InvalidIteratorError):
        qeval("x in new Foo()")

    class Bar(list):
        def sum(self):
            return sum(self)

    polar.register_class(Bar)
    assert qvar("x in new Bar([1, 2, 3])", "x") == [1, 2, 3]
    assert qvar("x = new Bar([1, 2, 3]).sum()", "x", one=True) == 6


def test_unexpected_expression(polar):
    """Ensure expression type raises error from core."""
    polar.load_str("f(x) if x > 2;")

    with pytest.raises(exceptions.UnexpectedPolarTypeError):
        next(polar.query_rule("f", Variable("x")))


def test_lookup_in_head(polar, is_allowed):
    # Test with enums
    class Actions(Enum):
        READ = 1
        WRITE = 2

    polar.register_class(Actions, name="Actions")
    polar.load_str('allow("leina", Actions.READ, "doc");')

    assert not is_allowed("leina", Actions.WRITE, "doc")
    assert not is_allowed("leina", "READ", "doc")
    assert not is_allowed("leina", 1, "doc")
    assert not is_allowed("leina", Actions, "doc")
    assert is_allowed("leina", Actions.READ, "doc")

    polar.clear_rules()

    # Test lookup in specializer raises error
    with pytest.raises(exceptions.UnrecognizedToken):
        polar.load_str('allow("leina", action: Actions.READ, "doc");')

    polar.clear_rules()

    # Test with normal class
    class Resource:
        def __init__(self, action):
            self.action = action

    polar.register_class(Resource, name="MyResource")
    polar.load_str('allow("leina", resource.action, resource: MyResource);')

    r = Resource("read")

    assert not is_allowed("leina", "write", r)
    assert is_allowed("leina", "read", r)


def test_isa_with_path(polar, query):
    @dataclass
    class Foo:
        num: int

    @dataclass
    class Bar:
        foo: Foo

    @dataclass
    class Baz:
        bar: Bar

    polar.register_class(Foo, fields={"num": int})
    polar.register_class(Bar, fields={"foo": Foo})
    polar.register_class(Baz, fields={"bar": Bar})

    polar.load_str(
        """
        f(x: Integer) if x = 0;
        g(x: Baz) if f(x.bar.foo.num);
        h(x: Bar) if f(x.num);
    """
    )
    results = query("g(x)", accept_expression=True)
    assert len(results) == 1

    with pytest.raises(exceptions.PolarRuntimeError):
        query("h(x)")


def test_rule_types_with_subclass_check(polar):
    class Foo:
        pass

    class Bar(Foo):
        pass

    class Baz(Bar):
        pass

    class Bad:
        pass

    # NOTE: keep this order of registering classes--confirms that MROs are added at the correct time
    polar.register_class(Baz)
    polar.register_class(Bar)
    polar.register_class(Foo)
    polar.register_class(Bad)

    p = """
    type f(_x: Integer);
    f(1);
    """
    polar.load_str(p)
    polar.clear_rules()

    p += """
    type f(_x: Foo);
    type f(_x: Foo, _y: Bar);
    f(_x: Bar);
    f(_x: Baz);
    """
    polar.load_str(p)
    polar.clear_rules()

    with pytest.raises(ValidationError):
        p += "f(_x: Bad);"
        polar.load_str(p)

    # Test with fields
    p = """
    type f(_x: Foo{id: 1});
    f(_x: Bar{id: 1});
    f(_x: Baz{id: 1});
    """
    polar.load_str(p)
    polar.clear_rules()

    with pytest.raises(ValidationError):
        p += "f(_x: Baz);"
        polar.load_str(p)

    # Test invalid rule type
    p = """
    type f(x: Foo, x.baz);
    """
    with pytest.raises(ValidationError):
        polar.load_str(p)


def test_unbound_dot_lookups(polar, is_allowed):
    """Port of GK's JS dot lookup test to Python"""

    @dataclass
    class Repo:
        id: int
        org_id: int

    @dataclass
    class Org:
        id: int

    @dataclass
    class Role:
        org_id: int

    @dataclass
    class User:
        roles: List[Role]

    repo1 = Repo(id=1, org_id=1)
    repo2 = Repo(id=2, org_id=2)
    user = User([Role(org_id=1)])

    for cls in [Repo, Org, Role, User]:
        polar.register_class(cls)

    polar.load_str(
        """
        user_in_role(user: User, "reader", org: Org) if
            role in user.roles and
            role.org_id = org.id;
        allow(user: User, "read", repo: Repo) if
            user_in_role(user, "reader", org) and
            repo.org_id = org.id;
    """
    )

    with pytest.raises(exceptions.PolarRuntimeError):
        assert is_allowed(user, "read", repo1)

    with pytest.raises(exceptions.PolarRuntimeError):
        assert not is_allowed(user, "read", repo2)
