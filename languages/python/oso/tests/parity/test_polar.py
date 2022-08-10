# Various polar tests. More integration less unit. Could move this into the module
# later if we wish to.
# Main things to test.
# - Direct tests of polar code.
# - Property tests for polar parser.
# - Property tests for polar semantics.
# - External functions / python binding tests of some kind, maybe.
import os
from pathlib import Path

import pytest

from polar.exceptions import PolarRuntimeError

from .test_polar_externals import Bar, Foo, MyClass, OurClass, Qux, YourClass

EXPECT_XFAIL_PASS = not bool(os.getenv("EXPECT_XFAIL_PASS", False))


@pytest.fixture
def externals(polar):
    polar.register_class(Qux)
    polar.register_class(Bar)
    polar.register_class(Foo)
    polar.register_class(MyClass)
    polar.register_class(YourClass)
    polar.register_class(OurClass)


def test_load_file(polar, qeval, qvar):
    polar.load_file(Path(__file__).parent / "policies/test.polar")
    assert qeval('test("true")')
    assert qvar("a(x)", "x", one=True) == "foo"


def test_query_multiple(polar, qvar):
    polar.load_str('a("foo"); a("bar"); a("baz");')
    results = qvar("a(x)", "x")
    assert results == ["foo", "bar", "baz"]


def test_define_rule(polar, qeval):
    polar.load_str(
        """a(x) if b(x) and c(x);
           b("apple");
           c("apple");"""
    )
    assert qeval('a("apple")')


def test_missing_rule(polar, qeval):
    polar.load_str(
        """a(x) if b(x) and c(x);
           b("apple");
           c("apple");"""
    )
    with pytest.raises(PolarRuntimeError) as e:
        qeval('d("apple")')
    assert "Query for undefined rule `d`" in str(e.value)


def test_negation(polar, qeval):
    polar.load_str('b("apple");')
    assert qeval('b("apple")')
    assert not qeval('not (b("apple"))')
    assert qeval('not (b("notanapple"))')


def test_recursive_rule(polar, qeval, qvar):
    polar.load_str(
        """derive("apple", "orange");
           derive("orange", "avacado");
           derive("avacado", "juniper_berry");
           derives(a, b) if derive(a, b);
           derives(a, b) if derive(a, z) and derives(z, b);"""
    )
    results = qvar('derive("apple", x)', "x")
    assert results == ["orange"]

    assert qeval('derives("apple", "juniper_berry")')

    results = qvar('derives("apple", x)', "x")
    assert results == ["orange", "avacado", "juniper_berry"]


def test_disjunctive_rule(polar, qeval):
    polar.load_str(
        """or_eq(a, b) if 1 = 0 or a = b;
           and_or_eq(a, b, c) if (a = b and b = c) or 1 = 0;"""
    )
    assert qeval("or_eq(1, 1)")

    assert not qeval("and_or_eq(1, 1, 0)")
    assert qeval("and_or_eq(1, 1, 1)")

    assert qeval("1=0 or (1=1 and 1=1)")
    assert not qeval("1=0 or (1=0 and 1=1)")

    # not sure if these test anything but :)
    assert qeval("1=0 or (1=0 or 1=1)")
    assert not qeval("1=0 or (1=0 or 1=0)")

    assert qeval("1=1 and (1=0 or 1=1)")
    assert not qeval("1=0 and (1=0 or 1=1)")


def test_parens(polar, qeval):
    polar.load_str(
        """paren1(a, b, c) if (a = b and b = c);
           paren2(a, b, c) if ((a = b and b = c));
           paren3(a, b, c) if (a = b) and (b = c);
           paren4(a, b, c, d) if (a = b and b = c and c = d);
           paren5(a, b, c) if ((a = b) and (b = c));"""
    )

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


def test_defining_things(polar, qeval):
    facts = [
        'apple("orange");',
        'thing("with", "two");',
        'thing("with", "a", "lot", "of", "arguments", 1, 2, 3, 4, 5);',
        'embedded("strings");',
    ]
    for f in facts:
        polar.load_str(f)
        assert qeval(f[:-1])
        polar.clear_rules()


def test_dictionaries(polar, qeval, qvar):
    # *** basic dictionary lookup ***
    polar.load_str('dict({hello: "world", foo: "bar"});')
    assert qeval('dict(d) and d.hello = "world"')

    polar.clear_rules()

    # dictionary lookups with variable fields ###
    polar.load_str("attr(d, k, d.(k));")

    # k = "hello", {hello: "steve"}.(k) = "steve"
    assert qeval('attr({hello: "steve"}, "hello", "steve")')

    # k = "hello", {hello: "steve"}.(k) = value, value = "steve"
    assert qvar('attr({hello: "steve"}, "hello", value)', "value", one=True) == "steve"

    # k = key, {hello: "steve"}.(k) = "steve", key = "hello"
    assert qvar('attr({hello: "steve"}, key, "steve")', "key", one=True) == "hello"

    # *** nested lookups ***
    assert qeval(
        'attr({hello: {this: {is: "nested"}}}, "hello", {this: {is: "nested"}})'
    )

    polar.clear_rules()

    polar.load_str("deepget(d, d.hello.this.is);")
    assert qeval('deepget({hello: {this: {is: "nested"}}}, "nested")')

    polar.clear_rules()

    polar.load_str("myget(d, d.get.inner);")
    assert qeval('myget({get: {inner: "nested"}}, "nested")')

    polar.clear_rules()

    polar.load_str(
        """x({a: {b:{c:123}}});
           x({a: {y:{c:456}}});"""
    )
    assert qvar("x(d) and d.a.(k).c = value", "value") == [123, 456]

    polar.clear_rules()

    polar.load_str("lookup(dict, result) if result = dict.a.b.c;")
    assert qeval('lookup({a: {b: {c: "nested"}}}, "nested")')

    polar.clear_rules()

    # *** more basic lookup tests ***
    polar.load_str(
        """user({name: "steve", job: "programmer", state: "NY"});
           user({name: "alex", job: "programmer", state: "CO"});
           user({name: "graham", job: "business", state: "NY"});"""
    )
    assert qeval('user(d) and d.name = "steve"')
    assert qvar('user({job: "programmer", name: name, state: state})', "name") == [
        "steve",
        "alex",
    ]


def test_external_classes(qeval, externals):
    assert qeval("new Bar() matches Foo")
    assert not qeval("new Qux() matches Foo")
    assert qeval('new Foo().foo() = "Foo!"')
    assert qeval('new Bar().foo() = "Bar!"')


@pytest.mark.xfail(
    reason="Doesn't work right now since we don't implement external instance unification."
)
def test_unify_class_fields(polar, qeval):
    polar.load_str("check(name, new Foo(name: name));")

    assert qeval('check("sam", new Foo(name: "sam"))')
    assert not qeval('check("alex", new Foo(name: "sam"))')


def test_argument_patterns(polar, qeval, externals):
    polar.load_str("matchesFoo(name, foo: Foo) if name = foo.name;")

    assert qeval('matchesFoo(sam, new Foo(name: "sam"))')
    assert qeval('matchesFoo(sam, new Bar(name: "sam"))')
    assert not qeval('matchesFoo("alex", new Foo(name: "sam"))')
    assert not qeval('matchesFoo("alex", new Bar(name: "sam"))')
    assert not qeval('matchesFoo("alex", new Qux())')


@pytest.mark.skip(reason="No longer support external instance unification")
# TODO: update to use internal classes (depends on instantiation bug fix)
def test_keys_are_confusing(qeval):
    assert qeval("new MyClass(x: 1, y: 2) = new MyClass(y: 2, x: 1)")
    assert qeval("new MyClass(x: 1, y: 2) = new MyClass(x: 1, y: 2)")
    assert not qeval("new MyClass(x: 1, y: 2) = new MyClass(x: 2, y: 1)")
    assert not qeval("new MyClass(x: 1, y: 2) = new MyClass(y: 1, x: 2)")
    assert not qeval("new MyClass(x: 1) = new MyClass(x: 1, y: 2)")
    assert not qeval("new MyClass(x: 1, y: 2) = new MyClass(y: 2)")


def test_matches(qeval, externals):
    assert qeval("{} matches {}")
    assert qeval("{x: 1} matches {}")
    assert qeval("{x: 1} matches {x: 1}")
    assert qeval("{x: 1, y: 2} matches {x: 1}")
    assert qeval("{x: 1, y: 2} matches {x: 1, y: 2}")
    assert qeval("{a: {x: 1, y: 2}} matches {a: {y: 2}}")
    assert not qeval("{a: {x: 1, y: 2}} matches {b: {y: 2}}")
    assert not qeval("{x: 1} matches {x: 1, y: 2}")
    assert not qeval("{y: 2} matches {x: 1, y: 2}")
    assert not qeval("{} matches {x: 1, y: 2}")
    assert not qeval("{} matches {x: 1}")

    assert qeval("new MyClass(x: 1, y: 2) matches {}")
    assert qeval("new MyClass(x: 1, y: 2) matches {x: 1, y: 2}")
    assert not qeval("{x: 1, y: 2} matches MyClass{x: 1, y: 2}")

    assert qeval("new MyClass(x: 1, y: 2) matches MyClass{x: 1}")
    assert not qeval("new MyClass(y: 2) matches MyClass{x: 1, y: 2}")

    assert qeval("new OurClass(x: 1, y: 2) matches YourClass")
    assert qeval("new OurClass(x: 1, y: 2) matches MyClass{x: 1}")
    assert qeval("new OurClass(x: 1, y: 2) matches MyClass{x: 1, y: 2}")
    assert not qeval("new MyClass(x: 1, y: 2) matches OurClass{x: 1}")
    assert not qeval("new MyClass(x: 1, y: 2) matches YourClass")
    assert not qeval("new MyClass(x: 1, y: 2) matches YourClass{}")


def test_nested_matches(qeval, externals):
    assert qeval(
        "new MyClass(x: new MyClass(x: 1, y: 2), y: 2) matches MyClass{x: MyClass{x: 1}}"
    )
    assert not qeval(
        "new MyClass(x: new MyClass(x: 1), y: 2) matches MyClass{x: MyClass{y: 2}}"
    )


def test_field_unification(qeval):
    # test dictionary field unification
    assert qeval("{} = {}")
    assert qeval("{x: 1} = {x: 1}")
    assert not qeval("{x: 1} = {x: 2}")
    assert not qeval("{x: 1} = {y: 1}")
    assert not qeval("{x: 1, y: 2} = {y: 1, x: 2}")
    assert qeval("{x: 1, y: 2} = {y: 2, x: 1}")


def test_field_unification_external(qeval, externals):
    # test instance field unification
    assert qeval("new MyClass(x: 1, y: 2) = new MyClass(y: 2, x: 1)")
    assert not qeval("new MyClass(x: 1, y: 2) = {y: 2, x: 1}")
    assert qeval("new MyClass(x: 1, y: 2) = new OurClass(y: 2, x: 1)")


@pytest.mark.xfail(EXPECT_XFAIL_PASS, reason="Internal classes not implemented yet.")
def test_class_definitions(qeval, polar):
    # Contains test queries.
    polar.load_file(Path(__file__).parent / "policies/classes.pol")

    # Test instantiation errors.
    with pytest.raises(PolarRuntimeError):
        qeval("NotADefinedClassName{foo: 1}")
    with pytest.raises(PolarRuntimeError):
        qeval("Three{foo: One{}}")
    with pytest.raises(PolarRuntimeError):
        qeval("Three{unit: Two{}}")
    with pytest.raises(PolarRuntimeError):
        qeval("Three{unit: One{}, pair: One{}}")


@pytest.mark.xfail(EXPECT_XFAIL_PASS, reason="Classes not implemented yet.")
def test_field_specializers(polar, qvar):
    # Contains test queries.
    polar.load_file(Path(__file__).parent / "policies/people.pol")

    # Test method ordering w/field specializers.
    assert qvar('froody(Manager{name: "Sam"}, x)', "x") == [1]
    assert qvar('froody(Manager{name: "Sam", id: 1}, x)', "x") == [2, 1]
    assert qvar(
        'froody(Manager{name: "Sam", id: 1, manager: Person{name: "Sam"}}, x)', "x"
    ) == [3, 2, 1]


@pytest.mark.xfail(EXPECT_XFAIL_PASS, reason="Groups not implemented yet.")
def test_groups(polar, qeval, query):
    # Contains test queries.
    polar.load_file(Path(__file__).parent / "policies/groups.pol")

    # Check that we can't instantiate groups.
    with pytest.raises(PolarRuntimeError):
        qeval("G{}")

    # Test rule ordering with groups.
    results = query("check_order(A{}, action)")
    expected = ["A", "G", "H"]
    assert expected == [result["action"] for result in results]


# TODO: Fix with
# https://www.notion.so/osohq/Internal-classes-cannot-be-instantiated-9554c7298feb4842b5448e7edf1d8b8b
@pytest.mark.skip("Skipped because of bug in class init.")
def test_group_field_access(polar, qvar):
    polar.load_file(Path(__file__).parent / "policies/groups.pol")

    assert qvar('get_bar(Baz{bar: "test"}, val)', "val", one=True) == "test"


def test_booleans(qeval):
    assert qeval("true = true")
    assert qeval("false = false")
    assert not qeval("true = false")


def test_comparisons(qeval):
    assert qeval("3 == 3")
    assert qeval("3 != 2")
    assert qeval("2 <= 2")
    assert qeval("2 <= 3")
    assert qeval("2 < 3")
    assert qeval("3 >= 3")
    assert qeval("3 >= 2")
    assert qeval("3 > 2")
    assert qeval("x = 1 and x == 1")


def test_bool_from_external_call(polar, qeval, qvar):
    class Booler:
        def whats_up(self):
            return True

    polar.register_class(Booler)

    assert qvar("new Booler().whats_up() = var", "var", one=True)
    assert qeval("new Booler().whats_up()")


def test_numbers_from_external_call(polar, qeval, qvar):
    class Numberer:
        def give_me_an_int(self):
            return 1

        def give_me_a_float(self):
            return 1.234

    polar.register_class(Numberer)

    result = qvar("new Numberer().give_me_an_int() = var", "var", one=True)
    assert result == 1
    assert qeval("new Numberer().give_me_an_int() = 1")

    result = qvar("new Numberer().give_me_a_float() = var", "var", one=True)
    assert result == 1.234
    assert qeval("new Numberer().give_me_a_float() = 1.234")


def test_arities(polar, qeval):
    polar.load_str("f(1); f(_x, _y);")
    assert qeval("f(1)")
    assert not qeval("f(2)")
    assert qeval("f(2, 3)")


def test_rule_ordering(polar, qeval, externals):
    polar.load_str(
        """f(_: Foo{}) if cut and 1 = 2;
           f(_: Foo{name: "test"});"""
    )

    assert qeval('f(new Foo( name: "test" )) ')
    assert qeval('x = new Foo( name: "test" ) and f(x) ')
    assert not qeval('f(new Foo( name: "nope" )) ')
    assert not qeval('x = new Foo( name: "nope" ) and f(x) ')
