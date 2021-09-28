import pytest

from oso import Oso, Relation
from helpers import check_authz, unord_eq, filter_array
from dataclasses import dataclass


@dataclass
class Bar:
    id: str
    is_cool: bool
    is_still_cool: bool

    def foos(self):
        return [foo for foo in foos if foo.bar_id == self.id]


@dataclass
class Foo:
    id: str
    bar_id: str
    is_fooey: bool
    numbers: list

    def bar(self):
        one_bar = [bar for bar in bars if bar.id == self.bar_id]
        assert len(one_bar) == 1
        return one_bar[0]

    def logs(self):
        return [log for log in logs if self.id == log.foo_id]


@dataclass
class Log:
    id: str
    foo_id: str
    data: str

    def foo(self):
        one_foo = [foo for foo in foos if foo.id == self.foo_id]
        assert len(one_foo) == 1
        return one_foo[0]


hello_bar = Bar(id="hello", is_cool=True, is_still_cool=True)
goodbye_bar = Bar(id="goodbye", is_cool=False, is_still_cool=True)
hershey_bar = Bar(id="hershey", is_cool=False, is_still_cool=False)

something_foo = Foo(id="something", bar_id="hello", is_fooey=False, numbers=[])
another_foo = Foo(id="another", bar_id="hello", is_fooey=True, numbers=[1])
third_foo = Foo(id="third", bar_id="hello", is_fooey=True, numbers=[2])
fourth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True, numbers=[2, 1])

fourth_log_a = Log(id="a", foo_id="fourth", data="hello")
third_log_b = Log(id="b", foo_id="third", data="world")
another_log_c = Log(id="c", foo_id="another", data="steve")

bars = [hello_bar, goodbye_bar, hershey_bar]
foos = [something_foo, another_foo, third_foo, fourth_foo]
logs = [fourth_log_a, third_log_b, another_log_c]


@pytest.fixture
def oso():
    oso = Oso()

    def get_bars(constraints):
        return filter_array(bars, constraints)

    def get_foos(constraints):
        return filter_array(foos, constraints)

    def get_foo_logs(constraints):
        return filter_array(logs, constraints)

    # Combining is combining but filtering out duplicates.
    def combine_query(q1, q2):
        results = q1 + q2
        return [i for n, i in enumerate(results) if i not in results[:n]]

    oso.set_data_filtering_query_defaults(
        exec_query=lambda results: results, combine_query=combine_query
    )

    oso.register_class(
        Bar,
        fields={
            "id": str,
            "is_cool": bool,
            "is_still_cool": bool,
            "foos": Relation(
                kind="many", other_type="Foo", my_field="id", other_field="bar_id"
            ),
        },
        build_query=get_bars,
    )
    oso.register_class(
        Foo,
        fields={
            "id": str,
            "bar_id": str,
            "is_fooey": bool,
            "numbers": list,
            "bar": Relation(
                kind="one", other_type="Bar", my_field="bar_id", other_field="id"
            ),
            "logs": Relation(
                kind="many",
                other_type="Log",
                my_field="id",
                other_field="foo_id",
            ),
        },
        build_query=get_foos,
    )
    oso.register_class(
        Log,
        fields={
            "id": str,
            "foo_id": str,
            "data": str,
            "foo": Relation(
                kind="one", other_type="Foo", my_field="foo_id", other_field="id"
            ),
        },
        build_query=get_foo_logs,
    )
    return oso


# cf. test_flask_model
def test_model(oso):
    oso.load_str(
        """
        allow("gwen", "get", foo: Foo) if foo.id = "something";
    """
    )
    check_authz(oso, "gwen", "get", Foo, [something_foo])

    oso.clear_rules()
    oso.load_str(
        """
        allow("gwen", "get", foo: Foo) if foo.id = "something";
        allow("gwen", "get", foo: Foo) if foo.id = "another";
    """
    )
    check_authz(oso, "gwen", "get", Foo, [another_foo, something_foo])


def test_authorize_scalar_attribute_eq(oso):
    oso.load_str(
        """
        allow(_: Bar, "read", foo: Foo) if
            foo.is_fooey;
        allow(bar: Bar, "read", foo: Foo) if
            foo.bar = bar;
    """
    )
    for bar in bars:
        expected = [foo for foo in foos if foo.is_fooey or foo.bar() is bar]
        check_authz(oso, bar, "read", Foo, expected)


def test_authorize_scalar_attribute_condition(oso):
    oso.load_str(
        """
        allow(bar: Bar, "read", foo: Foo) if
            foo.bar.is_cool = true and
            foo.bar.id = bar.id;
        allow(_: Bar, "read", foo: Foo) if
            foo.bar.is_cool = true and
            foo.is_fooey = true;
        allow(bar: Bar, "read", foo: Foo) if
            foo.bar.is_cool = false and
            bar.is_still_cool = true;
    """
    )
    for bar in bars:
        expected = [
            foo
            for foo in foos
            if foo.bar().is_cool
            and foo.bar() is bar
            or foo.bar().is_cool
            and foo.is_fooey
            or not foo.bar().is_cool
            and bar.is_still_cool
        ]
        check_authz(oso, bar, "read", Foo, expected)


def test_in_multiple_attribute_relationship(oso):
    oso.load_str(
        """
        allow(_, "read", _: Foo{is_fooey: false});
        allow(bar, "read", _: Foo{bar: bar});
        allow(_, "read", foo: Foo) if
            num in foo.numbers and
            foo.bar.is_cool and
            num = 1;
        allow(_, "read", foo: Foo) if
            num in foo.numbers and
            foo.bar.is_cool and
            num = 2;
    """
    )

    for bar in bars:
        expected = [
            foo
            for foo in foos
            if not foo.is_fooey
            or foo.bar() is bar
            or foo.bar().is_cool
            and (1 in foo.numbers or 2 in foo.numbers)
        ]
        check_authz(oso, bar, "read", Foo, expected)


def test_nested_relationship_many_single(oso):
    oso.load_str(
        """
        allow(log: Log, "read", bar: Bar) if
            log.foo in bar.foos;
    """
    )
    for log in logs:
        expected = [bar for bar in bars if log.foo() in bar.foos()]
        check_authz(oso, log, "read", Bar, expected)


def test_nested_relationship_many_many(oso):
    oso.load_str(
        """
        allow(log: Log, "read", bar: Bar) if
            foo in bar.foos and
            log in foo.logs;
    """
    )
    for log in logs:
        expected = [bar for bar in bars for foo in bar.foos() if log in foo.logs()]
        check_authz(oso, log, "read", Bar, expected)


def test_nested_relationship_many_many_constrained(oso):
    oso.load_str(
        """
        allow(log: Log, "read", bar: Bar) if
            foo in bar.foos and
            log in foo.logs and
            log.data = "steve";
    """
    )
    for log in logs:
        expected = [
            bar
            for bar in bars
            for foo in bar.foos()
            if log in foo.logs() and log.data == "steve"
        ]
        if log.data == "steve":
            assert expected
        else:
            assert not expected
        check_authz(oso, log, "read", Bar, expected)


# TODO
# def test_nested_relationship_many_many_many_constrained(oso):


def test_partial_in_collection(oso):
    oso.load_str(
        """
        allow(bar, "read", foo: Foo) if foo in bar.foos;
    """
    )
    for bar in bars:
        check_authz(oso, bar, "read", Foo, bar.foos())


def test_empty_constraints_in(oso):
    oso.load_str(
        """
        allow(_, "read", foo: Foo) if _n in foo.logs;
    """
    )
    expected = [foo for foo in foos if foo.numbers]
    check_authz(oso, "gwen", "read", Foo, expected)


def test_in_with_constraints_but_no_matching_object(oso):
    oso.load_str(
        """
        allow(_, "read", foo: Foo) if 99 in foo.numbers;
    """
    )
    check_authz(oso, "gwen", "read", Foo, [])


def test_redundant_in_on_same_field(oso):
    # gwen can read any foo whose numbers include 1 and 2
    oso.load_str(
        """
        allow("gwen", "read", foo: Foo) if
            m in foo.numbers and
            n in foo.numbers and
            m = 1 and n = 2;
    """
    )

    expected = [foo for foo in foos if 1 in foo.numbers and 2 in foo.numbers]
    assert expected == [fourth_foo]
    check_authz(oso, "gwen", "read", Foo, expected)


def test_unify_ins(oso):
    # gwen can read any bar with at least one foo
    oso.load_str(
        """
        allow("gwen", "read", bar: Bar) if
            foo in bar.foos and
            goo in bar.foos and
            foo = goo;
    """
    )

    expected = [
        bar
        for bar in bars
        if [foo for foo in bar.foos() for goo in bar.foos() if foo is goo]
    ]
    assert unord_eq(expected, [hello_bar, goodbye_bar])
    check_authz(oso, "gwen", "read", Bar, expected)


@pytest.mark.xfail(reason="a bug")
def test_unify_ins_neq(oso):
    # gwen can read any bar with at least two foos
    oso.load_str(
        """
        allow(_, "read", bar: Bar) if
            foo in bar.foos and
            goo in bar.foos and
            foo != goo;
    """
    )

    expected = [
        bar
        for bar in bars
        if [foo for foo in bar.foos() for goo in bar.foos() if foo is not goo]
    ]
    check_authz(oso, "gwen", "read", Bar, expected)


@pytest.mark.xfail(reason="a bug")
def test_unify_ins_field_eq(oso):
    oso.load_str(
        """
        allow(_, "read", bar: Bar) if
            foo in bar.foos and
            goo in bar.foos and
            foo.id = goo.id;
    """
    )

    result = oso.authorized_resources("gwen", "read", Bar)
    assert len(result) == 2


@pytest.mark.xfail(reason="a bug")
def test_deeply_nested_in(oso):
    # gwen can read any foo whose bar has another foo.
    oso.load_str(
        """
        allow("gwen", "read", a: Foo) if
            b in a.bar.foos and b != a and
            c in b.bar.foos and c != b and
            d in c.bar.foos and d != c and
            e in d.bar.foos and e != d;
    """
    )

    result = oso.authorized_resources("gwen", "read", Foo)
    assert len(result) == 3


@pytest.mark.xfail(reason="a bug")
def test_in_intersection(oso):
    # gwen can read any foo with a sibling foo with a number in common
    oso.load_str(
        """
        allow("gwen", "read", foo: Foo) if
            num in foo.numbers and
            goo in foo.bar.foos and
            goo != foo and
            num in goo.numbers;
    """
    )
    result = oso.authorized_resources("gwen", "read", Foo)
    assert len(result) == 0


def test_partial_isa_with_path(oso):
    oso.load_str(
        """
        allow(_, _, foo: Foo) if check(foo.bar);
        check(bar: Bar) if bar.id = "goodbye";   # this should match
        check(foo: Foo) if foo.bar.id = "hello"; # this shouldn't match
    """
    )
    check_authz(oso, "gwen", "read", Foo, goodbye_bar.foos())


def test_no_relationships(oso):
    oso.load_str('allow("steve", "get", foo: Foo) if foo.is_fooey = true;')
    expected = [foo for foo in foos if foo.is_fooey]
    check_authz(oso, "steve", "get", Foo, expected)


def test_neq(oso):
    oso.load_str(
        """
        allow("steve", action, foo: Foo) if foo.bar.id != action;
    """
    )

    for bar in bars:
        expected = [foo for foo in foos if foo.bar() != bar]
        check_authz(oso, "steve", bar.id, Foo, expected)


def test_relationship(oso):
    oso.load_str(
        """
        allow("steve", "get", resource: Foo) if
            resource.bar = bar and
            bar.is_cool = true and
            resource.is_fooey = true;
    """
    )

    expected = [foo for foo in foos if foo.bar().is_cool and foo.is_fooey]
    assert another_foo in expected
    assert len(expected) == 2
    check_authz(oso, "steve", "get", Foo, expected)


def test_duplex_relationship(oso):
    oso.load_str("allow(_, _, foo: Foo) if foo in foo.bar.foos;")
    check_authz(oso, "gwen", "gwen", Foo, foos)


def test_scalar_in_list(oso):
    oso.load_str(
        """
        allow("steve", "get", resource: Foo) if
            resource.bar = bar and
            bar.is_cool in [true, false];
    """
    )
    check_authz(oso, "steve", "get", Foo, foos)


def test_var_in_var(oso):
    oso.load_str(
        """
        allow("steve", "get", resource: Foo) if
            log in resource.logs and
            log.data = "hello";
    """
    )
    expected = [foo for foo in foos for log in foo.logs() if log.data == "hello"]
    assert fourth_foo in expected
    check_authz(oso, "steve", "get", Foo, expected)


def test_parent_child_cases(oso):
    oso.load_str(
        """
        allow(log: Log, "A", foo: Foo) if
          log.foo = foo;
        allow(log: Log, "B", foo: Foo) if
          log in foo.logs;
        allow(log: Log, "C", foo: Foo) if
          log.foo = foo and log in foo.logs;
        allow(log: Log, "D", foo: Foo) if
          log in foo.logs and log.foo = foo;
    """
    )
    for action in ["A", "B", "C", "D"]:
        for log in logs:
            check_authz(oso, log, action, Foo, [log.foo()])


def test_specializers(oso):
    oso.load_str(
        """
        allow(foo: Foo,             "NoneNone", log) if foo = log.foo;
        allow(foo,                  "NoneCls",  log: Log) if foo = log.foo;
        allow(foo,                  "NoneDict", _: {foo:foo});
        allow(foo,                  "NonePtn",  _: Log{foo: foo});
        allow(foo: Foo,             "ClsNone",  log) if log in foo.logs;
        allow(foo: Foo,             "ClsCls",   log: Log) if foo = log.foo;
        allow(foo: Foo,             "ClsDict",  _: {foo: foo});
        allow(foo: Foo,             "ClsPtn",   _: Log{foo: foo});
        allow(_: {logs: logs},      "DictNone", log) if log in logs;
        allow(_: {logs: logs},      "DictCls",  log: Log) if log in logs;
        allow(foo: {logs: logs},    "DictDict", log: {foo: foo}) if log in logs;
        allow(foo: {logs: logs},    "DictPtn",  log: Log{foo: foo}) if log in logs;
        allow(_: Foo{logs: logs},   "PtnNone",  log) if log in logs;
        allow(_: Foo{logs: logs},   "PtnCls",   log: Log) if log in logs;
        allow(foo: Foo{logs: logs}, "PtnDict",  log: {foo: foo}) if log in logs;
        allow(foo: Foo{logs: logs}, "PtnPtn",   log: Log{foo: foo}) if log in logs;
    """
    )
    parts = ["None", "Cls", "Dict", "Ptn"]
    for a in parts:
        for b in parts:
            for log in logs:
                check_authz(oso, log.foo(), a + b, Log, [log])


def test_ground_object_in_collection(oso):
    # value in var
    oso.load_str(
        """
        allow("steve", "get", resource: Foo) if
            1 in resource.numbers and
            2 in resource.numbers;
    """
    )
    check_authz(oso, "steve", "get", Foo, [fourth_foo])


@pytest.mark.xfail(reason="not yet supported")
def test_forall_in_collection(oso):
    oso.load_str(
        "allow(_, _, bar: Bar) if forall(foo in bar.foos, foo.is_fooey = true);"
    )
    results = oso.authorized_resources("gwen", "get", Bar)
    assert len(results) == 3


@pytest.mark.xfail(reason="not yet supported")
def test_no_objects_collection_condition(oso):
    oso.load_str("allow(_, _, bar: Bar) if not (foo in bar.foos and foo.is_fooey);")
    results = oso.authorized_resources("gwen", "get", Bar)
    assert len(results) == 0


def test_var_in_value(oso):
    # @TODO(steve): There is maybe a way to optimize the filter plan where if we are doing
    # two different of the same fetch with different fields we can combine them into an `in`.

    # var in value, This currently doesn't come through as an `in`
    # This is I think the thing that MikeD wants though, for this to come through
    # as an in so the SQL can do an IN.
    oso.load_str(
        """
        allow("steve", "get", resource: Log) if
            resource.data in ["hello", "world"];
    """
    )
    expected = [log for log in logs if log.data in ["hello", "world"]]
    assert fourth_log_a in expected
    check_authz(oso, "steve", "get", Log, expected)


@pytest.mark.skip(
    """
    `or` constraints come from `not` negations and should instead be expanded in the
    simplifier"""
)
def test_or(oso):
    oso.load_str(
        """
        allow("steve", "get", r: Foo) if
            not (r.id = "something" and r.bar_id = "hello");
    """
    )

    results = oso.authorized_resources("steve", "get", Foo)
    assert len(results) == 2


def test_field_comparison(oso):
    oso.load_str(
        """
        allow("gwen", "eq", bar: Bar) if
            bar.is_cool = bar.is_still_cool;
        allow("gwen", "neq", bar: Bar) if
            bar.is_cool != bar.is_still_cool;
    """
    )
    expected = [b for b in bars if b.is_cool == b.is_still_cool]
    check_authz(oso, "gwen", "eq", Bar, expected)
    expected = [b for b in bars if b.is_cool != b.is_still_cool]
    check_authz(oso, "gwen", "neq", Bar, expected)


@pytest.mark.xfail(reason="doesn't work yet!")
def test_field_cmp_rel_field(oso):
    oso.load_str("allow(_, _, foo: Foo) if foo.bar.is_cool = foo.is_fooey;")
    expected = [foo for foo in foos if foo.is_fooey == foo.bar().is_cool]
    check_authz(oso, "gwen", "get", Foo, expected)


def test_const_in_coll(oso):
    magic = 1
    oso.register_constant(magic, "magic")
    oso.load_str(
        """
        allow(_, _, foo: Foo) if
            magic in foo.numbers;
    """
    )
    expected = [f for f in foos if magic in f.numbers]
    check_authz(oso, "gwen", "eat", Foo, expected)


@pytest.mark.xfail(reason="negation unsupported")
def test_const_not_in_coll(oso):
    magic = 1
    oso.register_constant(magic, "magic")
    oso.load_str(
        """
        allow(_, _, foo: Foo) if
            not (magic in foo.numbers);
    """
    )
    expected = [f for f in foos if magic not in f.numbers]
    check_authz(oso, "gwen", "eat", Foo, expected)


def test_param_field(oso):
    oso.load_str(
        """
        allow(actor, action, resource: Log) if
            actor = resource.data and
            action = resource.id;
    """
    )
    expected = [log for log in logs if log.data == "steve" and log.id == "c"]
    assert another_log_c in expected
    check_authz(oso, "steve", "c", Log, expected)
