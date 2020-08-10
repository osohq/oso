from datetime import datetime, timedelta
from math import pi
from pathlib import Path

from polar import polar_class
from polar import exceptions, Polar, Predicate, Query, Variable
from polar.test_helpers import db, polar, tell, load_file, query, qeval, qvar
from polar.exceptions import ParserException, PolarRuntimeException

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


def test_data_conversions(polar, qvar, query):
    polar.load_str('a(1);b("two");c(true);d([1,"two",true]);')
    assert qvar("a(x)", "x", one=True) == 1
    assert qvar("b(x)", "x", one=True) == "two"
    assert qvar("c(x)", "x", one=True)
    assert qvar("d(x)", "x", one=True) == [1, "two", True]


def test_load_function(polar, query, qvar):
    """Make sure the load function works."""
    # Loading the same file twice doesn't mess stuff up.
    polar.load_file(Path(__file__).parent / "test_file.polar")
    with pytest.raises(exceptions.PolarRuntimeException) as e:
        polar.load_file(Path(__file__).parent / "test_file.polar")
    assert (
        str(e.value)
        == f"File {Path(__file__).parent}/test_file.polar has already been loaded."
    )
    with pytest.raises(exceptions.PolarRuntimeException) as e:
        polar.load_file(Path(__file__).parent / "test_file_renamed.polar")
    assert (
        str(e.value)
        == f"A file with the same contents as {Path(__file__).parent}/test_file_renamed.polar named {Path(__file__).parent}/test_file.polar has already been loaded."
    )
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]

    polar.clear()
    polar.load_file(Path(__file__).parent / "test_file.polar")
    polar.load_file(Path(__file__).parent / "test_file_gx.polar")
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert query("g(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]


def test_external(polar, qvar):
    class Bar:
        def y(self):
            return "y"

    class Foo:
        def __init__(self, a="a"):
            self.a = a

        def b(self):
            yield "b"

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

    def capital_foo():
        return Foo(a="A")

    polar.register_class(Foo, from_polar=capital_foo)
    assert qvar("new Foo{}.a = x", "x", one=True) == "A"
    assert qvar("new Foo{}.a() = x", "x", one=True) == "A"
    assert qvar("new Foo{}.b = x", "x", one=True) == "b"
    assert qvar("new Foo{}.b() = x", "x", one=True) == "b"
    assert qvar("Foo.c = x", "x", one=True) == "c"
    assert qvar("Foo.c() = x", "x", one=True) == "c"
    assert qvar("new Foo{} = f and f.a() = x", "x", one=True) == "A"
    assert qvar("new Foo{}.bar().y() = x", "x", one=True) == "y"
    assert qvar("new Foo{}.e = x", "x", one=True) == [1, 2, 3]
    assert qvar("new Foo{}.f = x", "x") == [[1, 2, 3], [4, 5, 6], 7]
    assert qvar("new Foo{}.g.hello = x", "x", one=True) == "world"
    assert qvar("new Foo{}.h = x", "x", one=True) is True


def test_class_specializers(polar, qvar, qeval, query):
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
    test(_: A{});
    test(_: B{});

    try(_: B{}, res) if res = 2;
    try(_: C{}, res) if res = 3;
    try(_: A{}, res) if res = 1;
    """
    polar.load_str(rules)

    assert qvar("new A{}.a = x", "x", one=True) == "A"
    assert qvar("new A{}.x = x", "x", one=True) == "A"
    assert qvar("new B{}.a = x", "x", one=True) == "A"
    assert qvar("new B{}.b = x", "x", one=True) == "B"
    assert qvar("new B{}.x = x", "x", one=True) == "B"
    assert qvar("new C{}.a = x", "x", one=True) == "A"
    assert qvar("new C{}.b = x", "x", one=True) == "B"
    assert qvar("new C{}.c = x", "x", one=True) == "C"
    assert qvar("new C{}.x = x", "x", one=True) == "C"
    assert qvar("new X{}.x = x", "x", one=True) == "X"

    assert len(query("test(new A{})")) == 1
    assert len(query("test(new B{})")) == 2

    assert qvar("try(new A{}, x)", "x") == [1]
    assert qvar("try(new B{}, x)", "x") == [2, 1]
    assert qvar("try(new C{}, x)", "x") == [3, 2, 1]
    assert qvar("try(new X{}, x)", "x") == []


def test_dict_specializers(polar, qvar, qeval, query):
    class Animal:
        def __init__(self, species=None, genus=None, family=None):
            self.genus = genus
            self.species = species

    polar.register_class(Animal)

    rules = """
    what_is(_: {genus: "canis"}, res) if res = "canine";
    what_is(_: {species: "canis lupus", genus: "canis"}, res) if res = "wolf";
    what_is(_: {species: "canis familiaris", genus: "canis"}, res) if res = "dog";
    """
    polar.load_str(rules)

    wolf = 'new Animal{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog = 'new Animal{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine = 'new Animal{genus: "canis", family: "canidae"}'

    assert len(query(f"what_is({wolf}, res)")) == 2
    assert len(query(f"what_is({dog}, res)")) == 2
    assert len(query(f"what_is({canine}, res)")) == 1

    assert qvar(f"what_is({wolf}, res)", "res") == ["wolf", "canine"]
    assert qvar(f"what_is({dog}, res)", "res") == ["dog", "canine"]
    assert qvar(f"what_is({canine}, res)", "res") == ["canine"]


def test_class_field_specializers(polar, qvar, qeval, query):
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

    wolf = 'new Animal{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog = 'new Animal{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine = 'new Animal{genus: "canis", family: "canidae"}'
    canid = 'new Animal{family: "canidae"}'
    animal = "new Animal{}"

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


def test_specializers_mixed(polar, qvar, qeval, query):
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

    wolf = 'new Animal{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog = 'new Animal{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine = 'new Animal{genus: "canis", family: "canidae"}'

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

    with pytest.raises(exceptions.PolarException):
        p.load_str("g(1); ?= g(2);")


def test_parser_errors(polar):
    # IntegerOverflow
    rules = """
    f(a) if a = 18446744073709551616;
    """
    with pytest.raises(exceptions.IntegerOverflow) as e:
        polar.load_str(rules)
    assert (
        str(e.value)
        == "'18446744073709551616' caused an integer overflow at line 2, column 17"
    )

    # InvalidTokenCharacter
    rules = """
    f(a) if a = "this is not
    allowed";
    """
    with pytest.raises(exceptions.InvalidTokenCharacter) as e:
        polar.load_str(rules)
    assert (
        str(e.value)
        == "'\\n' is not a valid character. Found in this is not at line 2, column 29"
    )

    rules = """
    f(a) if a = "this is not allowed\0
    """

    with pytest.raises(exceptions.InvalidTokenCharacter) as e:
        polar.load_str(rules)
    assert (
        str(e.value)
        == "'\\u{0}' is not a valid character. Found in this is not allowed at line 2, column 17"
    )

    # InvalidToken -- not sure what causes this

    # UnrecognizedEOF
    rules = """
    f(a)
    """
    with pytest.raises(exceptions.UnrecognizedEOF) as e:
        polar.load_str(rules)
    assert (
        str(e.value)
        == "hit the end of the file unexpectedly. Did you forget a semi-colon at line 2, column 9"
    )

    # UnrecognizedToken
    rules = """
    1;
    """
    with pytest.raises(exceptions.UnrecognizedToken) as e:
        polar.load_str(rules)
    assert str(e.value) == "did not expect to find the token '1' at line 2, column 5"

    # ExtraToken -- not sure what causes this


def test_runtime_errors(polar, query):
    rules = """
    foo(a,b) if a in b;
    """
    polar.load_str(rules)
    with pytest.raises(exceptions.PolarRuntimeException) as e:
        query("foo(1,2)")
    assert (
        str(e.value)
        == """trace (most recent evaluation last):
  in query at line 1, column 1
    foo(1, 2)
  in rule foo at line 2, column 17
    _a_3 in _b_4
  in rule foo at line 2, column 17
    _a_3 in _b_4
Type error: can only use `in` on a list, this is Variable(Symbol("_a_3")) at line 2, column 17"""
    )


def test_lookup_errors(polar, query):
    class Foo:
        def foo(self):
            return "foo"

    polar.register_class(Foo)

    # Unify with an invalid field doesn't error.
    assert query('new Foo{} = {bar: "bar"}') == []
    # Dot op with an invalid field does error.
    with pytest.raises(exceptions.PolarRuntimeException) as e:
        query('new Foo{}.bar = "bar"') == []
    assert "Application error: 'Foo' object has no attribute 'bar'" in str(e.value)


def test_predicate(polar, qvar, query):
    """Test that predicates can be converted to and from python."""
    polar.load_str("f(x) if x = pred(1, 2);")
    assert qvar("f(x)", "x") == [Predicate("pred", [1, 2])]

    assert query(Predicate(name="f", args=[Predicate("pred", [1, 2])])) == [{}]


def test_return_list(polar, query):
    class Actor:
        def groups(self):
            return ["engineering", "social", "admin"]

    polar.register_class(Actor)

    # for testing lists
    polar.load_str('allow(actor: Actor, "join", "party") if "social" in actor.groups;')

    assert query(Predicate(name="allow", args=[Actor(), "join", "party"]))


def test_query(load_file, polar, query):
    """Test that queries work with variable arguments"""

    load_file(Path(__file__).parent / "test_file.polar")
    # plaintext polar query: query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]

    assert query(Predicate(name="f", args=[Variable("a")])) == [
        {"a": 1},
        {"a": 2},
        {"a": 3},
    ]


def test_constructor(polar, qvar):
    """Test that class constructor is called correctly with constructor syntax."""

    class TestConstructor:
        def __init__(self, x):
            self.x = x

    polar.register_class(TestConstructor)

    assert (
        qvar("instance = new TestConstructor{x: 1} and y = instance.x", "y", one=True)
        == 1
    )
    assert (
        qvar("instance = new TestConstructor{x: 2} and y = instance.x", "y", one=True)
        == 2
    )
    assert (
        qvar(
            "instance = new TestConstructor{x: new TestConstructor{x: 3}} and y = instance.x.x",
            "y",
            one=True,
        )
        == 3
    )

    class TestConstructorTwo:
        def __init__(self, x, y):
            self.x = x
            self.y = y

    polar.register_class(TestConstructorTwo)

    assert (
        qvar(
            "instance = new TestConstructorTwo{x: 1, y: 2} and x = instance.x and y = instance.y",
            "y",
            one=True,
        )
        == 2
    )


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


def test_in(polar, qeval):
    polar.load_str("g(x, y) if not x in y;")
    polar.load_str("f(x) if not (x=1 or x=2);")
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

    polar.load_str("foo() if new Foo{foo: 1} = new Foo{foo: 1};")
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

    polar.load_str("lt(a, b) if a < b;")
    polar.load_str("gt(a, b) if a > b;")
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

    # test creating datetime from polar
    polar.load_str("dt(x) if x = new Datetime{year: 2020, month: 5, day: 25};")
    assert query(Predicate("dt", [Variable("x")])) == [{"x": datetime(2020, 5, 25)}]
    polar.load_str("ltnow(x) if x < Datetime.now();")
    assert query(Predicate("ltnow", [t1]))
    assert not query(Predicate("ltnow", [t3]))

    polar.load_str(
        "timedelta(a: Datetime, b: Datetime) if a.__sub__(b) == new Timedelta{days: 1};"
    )
    assert query(Predicate("timedelta", [t4, t1]))


def test_other_constants(polar, qvar, query):
    d = {"a": 1}
    polar.register_constant("d", d)
    assert qvar("x = d.a", "x") == [1]


def test_host_methods(polar, qeval, query):
    assert qeval('x = "abc" and x.startswith("a") = true and x.find("bc") = 1')
    assert qeval("i = 4095 and i.bit_length() = 12")
    assert qeval('f = 3.14159 and f.hex() = "0x1.921f9f01b866ep+1"')
    assert qeval("l = [1, 2, 3] and l.index(3) = 2 and l.copy() = [1, 2, 3]")
    assert qeval('d = {a: 1} and d.get("a") = 1 and d.get("b", 2) = 2')


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


def test_unbound_variable(polar, query):
    """Test that unbound variable is returned."""
    polar.load_str("rule(x, y) if y = 1;")

    first = query("rule(x, y)")[0]

    # y will be bound to 1
    first["y"] = 1

    # x should be unbound
    assert isinstance(first["x"], Variable)


def test_return_none(polar, qeval):
    class Foo:
        def this_is_none(self):
            return None

    polar.register_class(Foo)
    polar.load_str("f(x) if x.this_is_none = 1;")
    assert not list(polar.query_rule("f", Foo()))

    polar.load_str("f(x) if x.this_is_none.bad_call = 1;")

    with pytest.raises(exceptions.PolarRuntimeException) as e:
        list(polar.query_rule("f", Foo()))
    assert str(e.value).find(
        "Application error: 'NoneType' object has no attribute 'bad_call'"
    )
