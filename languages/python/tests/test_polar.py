from pathlib import Path

from polar import Polar
from polar.test_helpers import db, polar, tell, load_file, query, qeval, qvar

import pytest


def test_anything_works():
    p = Polar()
    p.load_str("f(1);")
    results = list(p.query_str("f(x)"))
    assert results[0]["x"] == 1
    results = list(p.query_str("f(y)"))
    assert results[0]["y"] == 1
    del p


def test_helpers(polar, load_file, query, qeval, qvar):
    load_file(Path(__file__).parent / "test_file.polar")  # f(1);
    assert query("f(x)") == [{"x": 1}, {"x": 2}, {"x": 3}]
    assert qvar("f(x)", "x") == [1, 2, 3]


def test_data_conversions(polar, qvar):
    polar.load_str('a(1);b("two");c(true);d([1,"two",true]);')
    assert qvar("a(x)", "x", one=True) == 1
    assert qvar("b(x)", "x", one=True) == "two"
    assert qvar("c(x)", "x", one=True)
    assert qvar("d(x)", "x", one=True) == [1, "two", True]


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

    polar.register_python_class(Foo, from_polar=capital_foo)
    assert qvar("Foo{}.a = x", "x", one=True) == "A"
    assert qvar("Foo{}.a() = x", "x", one=True) == "A"
    assert qvar("Foo{}.b = x", "x", one=True) == "b"
    assert qvar("Foo{}.b() = x", "x", one=True) == "b"
    assert qvar("Foo{}.c = x", "x", one=True) == "c"
    assert qvar("Foo{}.c() = x", "x", one=True) == "c"
    assert qvar("Foo{} = f, f.a() = x", "x", one=True) == "A"
    assert qvar("Foo{}.bar().y() = x", "x", one=True) == "y"
    assert qvar("Foo{}.e = x", "x", one=True) == [1, 2, 3]
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

    polar.register_python_class(A)
    polar.register_python_class(B)
    polar.register_python_class(C)
    polar.register_python_class(X)

    rules = """
    test(A{});
    test(B{});

    try(v: B{}, res) := res = 2;
    try(v: C{}, res) := res = 3;
    try(v: A{}, res) := res = 1;
    """
    polar.load_str(rules)

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

    polar.register_python_class(Animal)

    rules = """
    what_is(animal: {genus: "canis"}, res) := res = "canine";
    what_is(animal: {species: "canis lupus", genus: "canis"}, res) := res = "wolf";
    what_is(animal: {species: "canis familiaris", genus: "canis"}, res) := res = "dog";
    """
    polar.load_str(rules)

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

    polar.register_python_class(Animal)

    rules = """
    what_is(animal: Animal{}, res) := res = "animal";
    what_is(animal: Animal{genus: "canis"}, res) := res = "canine";
    what_is(animal: Animal{family: "canidae"}, res) := res = "canid";
    what_is(animal: Animal{species: "canis lupus", genus: "canis"}, res) := res = "wolf";
    what_is(animal: Animal{species: "canis familiaris", genus: "canis"}, res) := res = "dog";
    what_is(animal: Animal{species: s, genus: "canis"}, res) := res = s;
    """
    polar.load_str(rules)

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

    polar.register_python_class(Animal)

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
    polar.load_str(rules)

    wolf = 'Animal{species: "canis lupus", genus: "canis", family: "canidae"}'
    dog = 'Animal{species: "canis familiaris", genus: "canis", family: "canidae"}'
    canine = 'Animal{genus: "canis", family: "canidae"}'
    canid = 'Animal{family: "canidae"}'
    animal = "Animal{}"

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
