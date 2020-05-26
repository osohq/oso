# Various polar tests. More integration less unit. Could move this into the module
# later if we wish to.
# Main things to test.
# - Direct tests of polar code.
# - Property tests for polar parser.
# - Property tests for polar semantics.
# - External functions / python binding tests of some kind, maybe.
from pathlib import Path

import os
import shutil
import tempfile

from polar.exceptions import PolarRuntimeException

from test_polar_externals import Qux, Bar, Foo, MyClass, YourClass, OurClass

try:
    # This import is required when running the rust version of the library
    # so that the fixture is registered with pytest.
    from polar.test_helpers import polar
except ImportError:
    pass

from polar.test_helpers import load_file, tell, query, qeval, qvar

import pytest


@pytest.fixture
def externals(polar):
    polar.register_python_class(Qux)
    polar.register_python_class(Bar)
    polar.register_python_class(Foo)
    polar.register_python_class(MyClass)
    polar.register_python_class(YourClass)
    polar.register_python_class(OurClass)


def test_load_file(load_file, tell, qeval, qvar):
    load_file(Path(__file__).parent / "policies/test.pol")
    assert qeval('test("true")')
    tell('b("foo")')
    assert qvar("a(x)", "x", one=True) == "foo"


def test_query_multiple(tell, qvar):
    tell('a("foo")')
    tell('a("bar")')
    tell('a("baz")')
    results = qvar("a(x)", "x")
    assert results == ["foo", "bar", "baz"]


def test_define_rule(tell, qeval):
    tell("a(x) := b(x), c(x);")
    tell('b("apple")')
    tell('c("apple")')
    assert qeval('a("apple")')


def test_missing_rule(tell, qeval):
    tell("a(x) := b(x), c(x);")
    tell('b("apple")')
    tell('c("apple")')
    assert not qeval('d("apple")')


def test_negation(tell, qeval):
    tell('b("apple")')
    assert qeval('b("apple")')
    assert not qeval('!(b("apple"))')
    assert qeval('!(b("notanapple"))')


def test_recursive_rule(tell, qeval, qvar):
    tell('derive("apple", "orange")')
    tell('derive("orange", "avacado")')
    tell('derive("avacado", "juniper_berry")')
    results = qvar('derive("apple", x)', "x")
    assert results == ["orange"]
    tell("derives(a, b) := derive(a, b);")
    tell("derives(a, b) := derive(a, z), derives(z, b);")
    assert qeval('derives("apple", "juniper_berry")')
    results = qvar('derives("apple", x)', "x")
    assert results == ["orange", "avacado", "juniper_berry"]


def test_disjunctive_rule(tell, qeval):
    tell("or_eq(a, b) := 1 = 0 | a = b;")
    assert qeval("or_eq(1, 1)")

    tell("and_or_eq(a, b, c) := (a = b, b = c) | 1 = 0")
    assert not qeval("and_or_eq(1, 1, 0)")
    assert qeval("and_or_eq(1, 1, 1)")

    assert qeval("1=0 | (1=1, 1=1)")
    assert not qeval("1=0 | (1=0, 1=1)")

    # not sure if these test anything but :)
    assert qeval("1=0 | (1=0 | 1=1)")
    assert not qeval("1=0 | (1=0 | 1=0)")

    assert qeval("1=1, (1=0 | 1=1)")
    assert not qeval("1=0, (1=0 | 1=1)")


def test_parens(tell, qeval):
    tell("paren1(a, b, c) := (a = b, b = c);")
    tell("paren2(a, b, c) := ((a = b, b = c));")
    tell("paren3(a, b, c) := (a = b), (b = c);")
    tell("paren4(a, b, c, d) := (a = b, b = c, c = d);")
    tell("paren5(a, b, c) := ((a = b), (b = c));")

    assert qeval("paren1(1, 1, 1)")
    assert not qeval("paren1(0, 1, 1)")
    assert not qeval("paren1(1, 1, 0)")
    assert not qeval("paren1(1, 0, 1)")

    assert qeval("paren2(1, 1, 1)")
    assert not qeval("paren2(0, 1, 1)")
    assert not qeval("paren2(1, 1, 0)")
    assert not qeval("paren2(1, 0, 1)")

    assert qeval("paren3(1, 1, 1)")
    assert not qeval("paren3(0, 1, 1)")
    assert not qeval("paren3(1, 1, 0)")
    assert not qeval("paren3(1, 0, 1)")

    assert qeval("paren4(1, 1, 1, 1)")
    assert not qeval("paren4(0, 1, 1, 1)")
    assert not qeval("paren4(1, 1, 0, 1)")
    assert not qeval("paren4(1, 1, 1, 0)")

    assert qeval("paren5(1, 1, 1)")
    assert not qeval("paren5(0, 1, 1)")
    assert not qeval("paren5(1, 1, 0)")
    assert not qeval("paren5(1, 0, 1)")


@pytest.mark.xfail(reason='thing(with("nested"), "stuff") fails')
def test_defining_things(tell, qeval):
    facts = [
        'apple("orange")',
        'thing("with", "two")',
        'thing("with", "a", "lot", "of", "arguments", 1, 2, 3, 4, 5)',
        'thing(with("nested"), "stuff")',
        "dream(within(a(dream(within(a(dream(within(a(_dream)))))))))",
        'embedded("strings")',
    ]
    for f in facts:
        tell(f)
    for f in facts:
        assert qeval(f)


@pytest.mark.xfail(reason="Does not parse.")
def test_dictionaries(tell, qeval, qvar):
    tell('{hello: "world", foo: "bar"}')
    tell('{hello: {this: {is: "nested"}}}')
    tell("attr(d, k, d.(k))")
    assert qeval('attr({hello: "steve"}, "hello", "steve")')
    assert qvar('attr({hello: "steve"}, "hello", value)', "value", one=True) == "steve"
    assert qvar('attr({hello: "steve"}, key, "steve")', "key", one=True) == "hello"

    assert qeval(
        'attr({hello: {this: {is: "nested"}}}, "hello", {this: {is: "nested"}})'
    )

    tell("deepget(d, d.hello.this.is)")
    assert qeval('deepget({hello: {this: {is: "nested"}}}, "nested")')

    tell("myget(d, d.get.in)")
    assert qeval('myget({get: {in: "nested"}}, "nested")')

    tell('user({name: "steve", job: "programmer", state: "NY"})')
    tell('user({name: "alex", job: "programmer", state: "CO"})')
    tell('user({name: "graham", job: "business", state: "NY"})')
    assert qeval('user(d), d.name = "steve"')
    assert qvar('user({job: "programmer", name: name, state: state})', "name") == [
        "steve",
        "alex",
    ]

    tell("x({a: {b:{c:123}}})")
    tell("x({a: {y:{c:456}}})")
    assert qvar("x(d), d.a.(k).c = value", "value") == [123, 456]


@pytest.mark.xfail(reason="isa(Bar{}, Foo{}) fails")
def test_external_classes(tell, qeval, qvar, externals):
    assert qeval("isa(Bar{}, Foo{})")
    assert not qeval("isa(Qux{}, Foo{})")
    assert qeval('Foo{}.foo = "Foo!"')
    assert qeval('Bar{}.foo = "Bar!"')


@pytest.mark.xfail(reason="Foo not registered.")
def test_unify_class_fields(tell, qeval, qvar):
    tell("check(name, Foo{name: name})")

    assert qeval('check("sam", Foo{name: "sam"})')
    assert not qeval('check("alex", Foo{name: "sam"})')


@pytest.mark.xfail(reason="Error calling name.")
def test_argument_patterns(tell, qeval, qvar, externals):
    tell("isaFoo(name, foo: Foo) := name = foo.name")

    assert qeval('isaFoo(sam, Foo{name: "sam"})')
    assert qeval('isaFoo(sam, Bar{name: "sam"})')
    assert not qeval('isaFoo("alex", Foo{name: "sam"})')
    assert not qeval('isaFoo("alex", Bar{name: "sam"})')
    assert not qeval('isaFoo("alex", Qux{})')


@pytest.mark.skip(reason="No longer support external instance unification")
# TODO: update to use internal classes (depends on instantiation bug fix)
def test_keys_are_confusing(tell, qeval, qvar, externals):
    assert qeval("MyClass{x: 1, y: 2} = MyClass{y: 2, x: 1}")
    assert qeval("MyClass{x: 1, y: 2} = MyClass{x: 1, y: 2}")
    assert not qeval("MyClass{x: 1, y: 2} = MyClass{x: 2, y: 1}")
    assert not qeval("MyClass{x: 1, y: 2} = MyClass{y: 1, x: 2}")
    assert not qeval("MyClass{x: 1} = MyClass{x: 1, y: 2}")
    assert not qeval("MyClass{x: 1, y: 2} = MyClass{y: 2}")


@pytest.mark.xfail(reason="isa({}, {}) fails on first line")
def test_isa(qeval, qvar, externals):
    assert qeval("isa({}, {})")
    assert qeval("isa({x: 1}, {})")
    assert qeval("isa({x: 1}, {x: 1})")
    assert qeval("isa({x: 1, y: 2}, {x: 1})")
    assert qeval("isa({x: 1, y: 2}, {x: 1, y: 2})")
    assert qeval("isa({a: {x: 1, y: 2}}, {a: {y: 2}})")
    assert not qeval("isa({a: {x: 1, y: 2}}, {b: {y: 2}})")
    assert not qeval("isa({x: 1}, {x: 1, y: 2})")
    assert not qeval("isa({y: 2}, {x: 1, y: 2})")
    assert not qeval("isa({}, {x: 1, y: 2})")
    assert not qeval("isa({}, {x: 1})")

    assert qeval("isa(MyClass{x: 1, y: 2}, {})")
    assert qeval("isa(MyClass{x: 1, y: 2}, {x: 1, y: 2})")
    assert not qeval("isa({x: 1, y: 2}, MyClass{x: 1, y: 2})")

    assert qeval("isa(MyClass{x: 1, y: 2}, MyClass{x: 1})")
    assert qeval(
        "isa(MyClass{x: MyClass{x: 1, y: 2}, y: 2}, MyClass{x: MyClass{x: 1}})"
    )
    assert not qeval("isa(MyClass{x: MyClass{x: 1}, y: 2}, MyClass{x: MyClass{y: 2}})")
    assert not qeval("isa(MyClass{y: 2}, MyClass{x: 1, y: 2})")

    assert qeval("isa(OurClass{x: 1, y: 2}, YourClass{})")
    assert qeval("isa(OurClass{x: 1, y: 2}, MyClass{x: 1})")
    assert qeval("isa(OurClass{x: 1, y: 2}, MyClass{x: 1, y: 2})")
    assert not qeval("isa(MyClass{x: 1, y: 2}, OurClass{x: 1})")
    assert not qeval("isa(MyClass{x: 1, y: 2}, YourClass{})")


@pytest.mark.xfail(reason="Field unification on instances fails without an exception")
def test_field_unification(qeval, externals):
    # test dictionary field unification
    assert qeval("{} = {}")
    assert qeval("{x: 1} = {x: 1}")
    assert not qeval("{x: 1} = {x: 2}")
    assert not qeval("{x: 1} = {y: 1}")
    assert not qeval("{x: 1, y: 2} = {y: 1, x: 2}")
    assert qeval("{x: 1, y: 2} = {y: 2, x: 1}")

    # test instance field unification (not allowed for external instances)
    with pytest.raises(PolarRuntimeException):
        assert qeval("MyClass{x: 1, y: 2} = MyClass{y: 2, x: 1}")
    # with pytest.raises(PolarRuntimeException):
    assert not qeval("MyClass{x: 1, y: 2} = {y: 2, x: 1}")
    with pytest.raises(PolarRuntimeException):
        assert not qeval("MyClass{x: 1, y: 2} = OurClass{y: 2, x: 1}")
    with pytest.raises(PolarRuntimeException):
        assert not qeval("MyClass{x: 1, y: 2} = YourClass{y: 2, x: 1}")


@pytest.mark.xfail(reason="Not implemented yet.")
def test_class_definitions(tell, qeval, load_file):
    # Contains test queries.
    load_file(Path(__file__).parent / "policies/classes.pol")

    # Test instantiation errors.
    with pytest.raises(PolarRuntimeException):
        qeval("NotADefinedClassName{foo: 1}")
    with pytest.raises(PolarRuntimeException):
        qeval("Three{foo: One{}}")
    with pytest.raises(PolarRuntimeException):
        qeval("Three{unit: Two{}}")
    with pytest.raises(PolarRuntimeException):
        qeval("Three{unit: One{}, pair: One{}}")


@pytest.mark.xfail(reason="Classes not implemented yet.")
def test_field_specializers(load_file, qvar):
    # Contains test queries.
    load_file(Path(__file__).parent / "policies/people.pol")

    # Test method ordering w/field specializers.
    assert qvar('froody(Manager{name: "Sam"}, x)', "x") == [1]
    assert qvar('froody(Manager{name: "Sam", id: 1}, x)', "x") == [2, 1]
    assert qvar(
        'froody(Manager{name: "Sam", id: 1, manager: Person{name: "Sam"}}, x)', "x"
    ) == [3, 2, 1]


@pytest.mark.xfail(reason="Groups not implemented yet.")
def test_groups(load_file, qeval, query):
    # Contains test queries.
    load_file(Path(__file__).parent / "policies/groups.pol")

    # Check that we can't instantiate groups.
    with pytest.raises(PolarRuntimeException):
        qeval("G{}")

    # Test rule ordering with groups.
    results = query("check_order(A{}, action)")
    expected = ["A", "G", "H"]
    assert expected == [result["action"] for result in results]


# TODO: Fix with
# https://www.notion.so/osohq/Internal-classes-cannot-be-instantiated-9554c7298feb4842b5448e7edf1d8b8b
@pytest.mark.skip("Skipped because of bug in class init.")
def test_group_field_access(load_file, qvar):
    load_file(Path(__file__).parent / "policies/groups.pol")

    assert qvar('get_bar(Baz{bar: "test"}, val)', "val", one=True) == "test"


@pytest.mark.xfail(reason="Booleans not implemented yet.")
def test_booleans(qeval):
    assert qeval("true = true")
    assert qeval("false = false")
    assert not qeval("true = false")


@pytest.mark.xfail(reason="panics.")
def test_comparisons(tell, qeval, qvar, query):
    assert qeval("3 == 3")
    assert qeval("3 != 2")
    assert qeval("2 <= 2")
    assert qeval("2 <= 3")
    assert qeval("2 < 3")
    assert qeval("3 >= 3")
    assert qeval("3 >= 2")
    assert qeval("3 > 2")
    assert qeval("x = 1, x == 1")


@pytest.mark.xfail(reason="type error")
def test_bool_from_external_call(polar, qeval, qvar, query):
    class Booler:
        def whats_up(self):
            yield True

    polar.register_python_class(Booler)

    result = qvar("Booler{}.whats_up() = var", "var", one=True)
    assert qeval("Booler{}.whats_up() = true")
