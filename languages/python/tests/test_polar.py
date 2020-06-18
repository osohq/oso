from pathlib import Path

from polar import exceptions, Polar, Predicate, Variable
from polar.test_helpers import db, polar, tell, load_file, query, qeval, qvar
from polar.exceptions import ParserException

import pytest


def test_anything_works():
    p = Polar()
    p._load_str("f(1);")
    results = list(p._query_str("f(x)"))
    assert results[0]["x"] == 1
    results = list(p._query_str("f(y)"))
    assert results[0]["y"] == 1
    del p


def test_helpers(polar, load_file, query, qeval, qvar):
    load_file(Path(__file__).parent / "test_file.polar")  # f(1);
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]


def test_data_conversions(polar, qvar):
    polar._load_str('a(1);b("two");c(true);d([1,"two",true]);')
    assert qvar("a(x)", "x", one=True) == 1
    assert qvar("b(x)", "x", one=True) == "two"
    assert qvar("c(x)", "x", one=True)
    assert qvar("d(x)", "x", one=True) == [1, "two", True]


def test_load_function(polar, query, qvar):
    """Make sure the load function works."""
    # Loading the same file twice doesn't mess stuff up.
    polar.load(Path(__file__).parent / "test_file.polar")
    polar.load(Path(__file__).parent / "test_file.polar")
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]

    polar.clear()
    polar.load(Path(__file__).parent / "test_file.polar")
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]
    polar.load(Path(__file__).parent / "test_file_gx.polar")
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert query("g(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]

    polar.clear()
    polar.load(Path(__file__).parent / "test_file.polar")
    polar.load(Path(__file__).parent / "test_file_gx.polar")
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

        def c(self):
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
    assert qvar("Foo{}.a = x", "x", one=True) == "A"
    assert qvar("Foo{}.a() = x", "x", one=True) == "A"
    assert qvar("Foo{}.b = x", "x", one=True) == "b"
    assert qvar("Foo{}.b() = x", "x", one=True) == "b"
    assert qvar("Foo{}.c = x", "x", one=True) == "c"
    assert qvar("Foo{}.c() = x", "x", one=True) == "c"
    assert qvar("Foo{} = f, f.a() = x", "x", one=True) == "A"
    assert qvar("Foo{}.bar().y() = x", "x", one=True) == "y"
    assert qvar("Foo{}.e = x", "x", one=True) == [1, 2, 3]  # returns an actual list
    assert qvar("Foo{}.f = x", "x") == [[1, 2, 3], [4, 5, 6], 7]
    assert qvar("Foo{}.g.hello = x", "x", one=True) == "world"
    assert qvar("Foo{}.h = x", "x", one=True) is True


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
    test(A{});
    test(B{});

    try(v: B{}, res) := res = 2;
    try(v: C{}, res) := res = 3;
    try(v: A{}, res) := res = 1;
    """
    polar._load_str(rules)

    assert qvar("A{}.a = x", "x", one=True) == "A"
    assert qvar("A{}.x = x", "x", one=True) == "A"
    assert qvar("B{}.a = x", "x", one=True) == "A"
    assert qvar("B{}.b = x", "x", one=True) == "B"
    assert qvar("B{}.x = x", "x", one=True) == "B"
    assert qvar("C{}.a = x", "x", one=True) == "A"
    assert qvar("C{}.b = x", "x", one=True) == "B"
    assert qvar("C{}.c = x", "x", one=True) == "C"
    assert qvar("C{}.x = x", "x", one=True) == "C"
    assert qvar("X{}.x = x", "x", one=True) == "X"

    assert len(query("test(A{})")) == 1
    assert len(query("test(B{})")) == 2

    assert qvar("try(A{}, x)", "x") == [1]
    assert qvar("try(B{}, x)", "x") == [2, 1]
    assert qvar("try(C{}, x)", "x") == [3, 2, 1]
    assert qvar("try(X{}, x)", "x") == []


def test_dict_specializers(polar, qvar, qeval, query):
    class Animal:
        def __init__(self, species=None, genus=None, family=None):
            self.genus = genus
            self.species = species

    polar.register_class(Animal)

    rules = """
    what_is(animal: {genus: "canis"}, res) := res = "canine";
    what_is(animal: {species: "canis lupus", genus: "canis"}, res) := res = "wolf";
    what_is(animal: {species: "canis familiaris", genus: "canis"}, res) := res = "dog";
    """
    polar._load_str(rules)

    wolf = 'Animal{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog = 'Animal{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine = 'Animal{genus: "canis", family: "canidae"}'

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
    what_is(animal: Animal{}, res) := res = "animal";
    what_is(animal: Animal{genus: "canis"}, res) := res = "canine";
    what_is(animal: Animal{family: "canidae"}, res) := res = "canid";
    what_is(animal: Animal{species: "canis lupus", genus: "canis"}, res) := res = "wolf";
    what_is(animal: Animal{species: "canis familiaris", genus: "canis"}, res) := res = "dog";
    what_is(animal: Animal{species: s, genus: "canis"}, res) := res = s;
    """
    polar._load_str(rules)

    wolf = 'Animal{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog = 'Animal{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine = 'Animal{genus: "canis", family: "canidae"}'
    canid = 'Animal{family: "canidae"}'
    animal = "Animal{}"

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
    what_is(animal: Animal{}, res) := res = "animal_class";
    what_is(animal: Animal{genus: "canis"}, res) := res = "canine_class";
    what_is(animal: {genus: "canis"}, res) := res = "canine_dict";
    what_is(animal: Animal{family: "canidae"}, res) := res = "canid_class";
    what_is(animal: {species: "canis lupus", genus: "canis"}, res) := res = "wolf_dict";
    what_is(animal: {species: "canis familiaris", genus: "canis"}, res) := res = "dog_dict";
    what_is(animal: Animal{species: "canis lupus", genus: "canis"}, res) := res = "wolf_class";
    what_is(animal: Animal{species: "canis familiaris", genus: "canis"}, res) := res = "dog_class";
    """
    polar._load_str(rules)

    wolf = 'Animal{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog = 'Animal{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine = 'Animal{genus: "canis", family: "canidae"}'

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
    p._load_str("f(1); f(2); ?= f(1); ?= !f(3);")

    with pytest.raises(exceptions.PolarException):
        p._load_str("g(1); ?= g(2);")


def test_parser_errors(polar):
    # IntegerOverflow
    rules = """
    f(a) := a = 18446744073709551616;
    """
    with pytest.raises(exceptions.IntegerOverflow) as e:
        polar._load_str(rules)
    assert (
        str(e.value)
        == "'18446744073709551616' caused an integer overflow at line 2, column 17"
    )

    # InvalidTokenCharacter
    rules = """
    f(a) := a = "this is not
    allowed";
    """
    with pytest.raises(exceptions.InvalidTokenCharacter) as e:
        polar._load_str(rules)
    assert (
        str(e.value)
        == "'\\n' is not a valid character. Found in this is not at line 2, column 29"
    )

    rules = """
    f(a) := a = "this is not allowed\0
    """

    with pytest.raises(exceptions.InvalidTokenCharacter) as e:
        polar._load_str(rules)
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
        polar._load_str(rules)
    assert (
        str(e.value)
        == "hit the end of the file unexpectedly. Did you forget a semi-colon at line 2, column 9"
    )

    # UnrecognizedToken
    rules = """
    1;
    """
    with pytest.raises(exceptions.UnrecognizedToken) as e:
        polar._load_str(rules)
    assert str(e.value) == "did not expect to find the token '1' at line 2, column 5"

    # ExtraToken -- not sure what causes this


def test_predicate(polar, qvar):
    """Test that predicates can be converted to and from python."""
    polar._load_str("f(x) := x = pred(1, 2);")
    assert qvar("f(x)", "x") == [Predicate("pred", [1, 2])]

    assert polar._query_pred(
        Predicate(name="f", args=[Predicate("pred", [1, 2])]), single=True
    ).results == [{}]


def test_return_list(polar):
    class Actor:
        def groups(self):
            return ["engineering", "social", "admin"]

    polar.register_class(Actor)

    # for testing lists
    polar._load_str('allow(actor: Actor, "join", "party") := "social" in actor.groups;')

    assert polar._query_pred(
        Predicate(name="allow", args=[Actor(), "join", "party"])
    ).success


def test_query(load_file, polar):
    """Test that queries work with variable arguments"""

    load_file(Path(__file__).parent / "test_file.polar")
    # plaintext polar query: query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]

    assert polar._query_pred(Predicate(name="f", args=[Variable("a")])).results == [
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
        qvar("instance = new TestConstructor{x: 1}, y = instance.x", "y", one=True) == 1
    )
    assert (
        qvar("instance = new TestConstructor{x: 2}, y = instance.x", "y", one=True) == 2
    )
    assert (
        qvar(
            "instance = new TestConstructor{x: new TestConstructor{x: 3}}, y = instance.x.x",
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
            "instance = new TestConstructorTwo{x: 1, y: 2}, x = instance.x, y = instance.y",
            "y",
            one=True,
        )
        == 2
    )


def test_in(polar, qeval):
    polar._load_str("g(x, y) := !x in y;")
    polar._load_str("f(x) := !(x=1 | x=2);")
    assert not qeval("f(1)")
    assert qeval("g(4, [1,2,3])")
    assert not qeval("g(1, [1,1,1])")
