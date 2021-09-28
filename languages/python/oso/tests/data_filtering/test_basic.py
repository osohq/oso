import pytest

from oso import Oso, Relation
from helpers import *
from basic_examples import *

# cf. test_flask_model
def test_model(oso, t):
    policy = 'allow("gwen", "get", foo: Foo) if foo.id = "something";'
    oso.load_str(policy)
    check_authz(oso, 'gwen', 'get', Foo, [t['something_foo']])

    policy = """
    allow("gwen", "get", foo: Foo) if foo.id = "something";
    allow("gwen", "get", foo: Foo) if foo.id = "another";
    """

    oso.clear_rules()
    oso.load_str(policy)
    check_authz(oso, 'gwen', 'get', Foo, [another_foo, something_foo])


def test_authorize_scalar_attribute_eq(oso, t):
    oso.load_str("""
        allow(_: Bar, "read", foo: Foo) if
            foo.is_fooey;
        allow(bar: Bar, "read", foo: Foo) if
            foo.bar = bar;
    """)
    results = oso.authorized_resources(hello_bar, 'read', Foo)
    assert len(results) == 4
    results = oso.authorized_resources(goodbye_bar, 'read', Foo)
    assert len(results) == 3

def test_authorize_scalar_attribute_condition(oso, t):
    oso.load_str("""
        allow(bar: Bar, "read", foo: Foo) if
            foo.bar.is_cool = true and
            foo.bar.id = bar.id;
        allow(_: Bar, "read", foo: Foo) if
            foo.bar.is_cool = true and
            foo.is_fooey = true;
        allow(bar: Bar, "read", foo: Foo) if
            foo.bar.is_cool = false and
            bar.is_still_cool = true;
    """)
    results = oso.authorized_resources(hello_bar, 'read', Foo)
    assert len(results) == 4
    results = oso.authorized_resources(goodbye_bar, 'read', Foo)
    assert len(results) == 3
    results = oso.authorized_resources(hershey_bar, 'read', Foo)
    assert len(results) == 2

def test_in_multiple_attribute_relationship(oso, t):
    oso.load_str("""
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
    """)

    results = oso.authorized_resources(hello_bar, 'read', Foo)
    assert len(results) == 3
    results = oso.authorized_resources(goodbye_bar, 'read', Foo)
    assert len(results) == 4
    results = oso.authorized_resources(hershey_bar, 'read', Foo)
    assert len(results) == 3

def test_nested_relationship_many_single(oso, t):
    oso.load_str("""
        allow(log: Log, "read", bar: Bar) if
            foo in bar.foos and
            log.foo_id = foo.id;
    """)
    for log in t['logs']:
        results = oso.authorized_resources(log, 'read', Bar)
        assert len(results) == 1


def test_nested_relationship_many_many(oso, t):
    oso.load_str("""
        allow(log: Log, "read", bar: Bar) if
            foo in bar.foos and
            log in foo.logs;
    """)
    for log in t['logs']:
        results = oso.authorized_resources(log, 'read', Bar)
        assert len(results) == 1


def test_nested_relationship_many_many_constrained(oso, t):
    oso.load_str("""
        allow(_, "read", bar: Bar) if
            foo in bar.foos and
            log in foo.logs and
            log.data = "steve";
    """)
    check_authz(oso, 'gwen', 'read', Bar, [hello_bar])

# TODO
# def test_nested_relationship_many_many_many_constrained(oso, t):

def test_partial_in_collection(oso, t):
    oso.load_str("""
        allow(bar, "read", foo: Foo) if foo in bar.foos;
    """)
    result = oso.authorized_resources(goodbye_bar, 'read', Foo)
    assert result == [fourth_foo]

def test_empty_constraints_in(oso, t):
    oso.load_str("""
        allow(_, "read", foo: Foo) if _n in foo.logs;
    """)

    result = oso.authorized_resources('gwen', 'read', Foo)
    assert len(result) == 3


def test_in_with_constraints_but_no_matching_object(oso, t):
    oso.load_str("""
        allow(_, "read", foo: Foo) if n in foo.numbers and n = 99;
    """)

    result = oso.authorized_resources('gwen', 'read', Foo)
    assert len(result) == 0


def test_redundant_in_on_same_field(oso, t):
    # gwen can read any foo whose numbers include 1 and 2
    oso.load_str("""
        allow("gwen", "read", foo: Foo) if
            m in foo.numbers and
            n in foo.numbers and
            m = 1 and n = 2;
    """)

    result = oso.authorized_resources('gwen', 'read', Foo)
    assert len(result) == 1


def test_unify_ins(oso, t):
    # gwen can read any bar with at least one foo
    oso.load_str("""
        allow("gwen", "read", bar: Bar) if
            foo in bar.foos and
            goo in bar.foos and
            foo = goo;
    """)

    result = oso.authorized_resources('gwen', 'read', Bar)
    assert len(result) == 2


@pytest.mark.xfail(reason="a bug")
def test_unify_ins_neq(oso, t):
    # gwen can read any bar with at least two foos
    oso.load_str("""
        allow(_, "read", bar: Bar) if
            foo in bar.foos and
            goo in bar.foos and
            foo != goo;
    """)

    result = oso.authorized_resources('gwen', 'read', Bar)
    assert len(result) == 1

@pytest.mark.xfail(reason="a bug")
def test_unify_ins_field_eq(oso, t):
    oso.load_str("""
        allow(_, "read", bar: Bar) if
            foo in bar.foos and
            goo in bar.foos and
            foo.id = goo.id;
    """)

    result = oso.authorized_resources('gwen', 'read', Bar)
    assert len(result) == 2

@pytest.mark.xfail(reason="a bug")
def test_deeply_nested_in(oso, t):
    # gwen can read any foo whose bar has another foo.
    oso.load_str("""
        allow("gwen", "read", a: Foo) if
            b in a.bar.foos and b != a and
            c in b.bar.foos and c != b and
            d in c.bar.foos and d != c and
            e in d.bar.foos and e != d;
    """)

    result = oso.authorized_resources('gwen', 'read', Foo)
    assert len(result) == 3


@pytest.mark.xfail(reason="a bug")
def test_in_intersection(oso, t):
    # gwen can read any foo with a sibling foo with a number in common
    oso.load_str("""
        allow("gwen", "read", foo: Foo) if
            num in foo.numbers and
            goo in foo.bar.foos and
            goo != foo and
            num in goo.numbers;
    """)
    result = oso.authorized_resources('gwen', 'read', Foo)
    assert len(result) == 0


def test_partial_isa_with_path(oso, t):
    oso.load_str("""
        allow(_, _, foo: Foo) if check(foo.bar);
        check(foo: Foo) if foo.bar.id = "hello";
        check(bar: Bar) if bar.id = "goodbye";
    """)

    result = oso.authorized_resources('gwen', 'read', Foo)
    assert len(result) == 1

def test_no_relationships(oso, t):
    # Write a policy
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.is_fooey = true;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", another_foo)

    results = oso.authorized_resources("steve", "get", Foo)
    assert len(results) == 3


def test_neq(oso, t):
    policy = """
    allow("steve", "get", foo: Foo) if foo.bar.id != "hello";
    allow("steve", "put", foo: Foo) if foo.bar.id != "goodbye";
    """
    oso.load_str(policy)
    check_authz(oso, "steve", "get", Foo, [fourth_foo])
    check_authz(
        oso,
        "steve",
        "put",
        Foo,
        [another_foo, third_foo, something_foo],
    )


def test_relationship(oso, t):
    oso.load_str("""
        allow("steve", "get", resource: Foo) if
            resource.bar = bar and
            bar.is_cool = true and
            resource.is_fooey = true;
    """)

    assert oso.is_allowed("steve", "get", another_foo)
    results = oso.authorized_resources("steve", "get", Foo)
    assert len(results) == 2


def test_duplex_relationship(oso, t):
    oso.load_str("allow(_, _, foo: Foo) if foo in foo.bar.foos;")
    check_authz(oso, "gwen", "gwen", t["Foo"], t["foos"])


@pytest.mark.skip(""" Cant filter non registered classes anymore.""")
def test_known_results(oso):
    oso.load_str("""
        allow(_, _, i: Integer) if i in [1, 2];
        allow(_, _, d: Dictionary) if d = {};
    """)

    results = oso.authorized_resources("gwen", "get", int)
    assert unord_eq(results, [1, 2])

    results = oso.authorized_resources("gwen", "get", dict)
    assert results == [{}]

    results = oso.authorized_resources("gwen", "get", str)
    assert results == []


def test_scalar_in_list(oso, t):
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.bar = bar and
        bar.is_cool in [true, false];
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", another_foo)

    results = list(oso.authorized_resources("steve", "get", Foo))
    assert len(results) == 4


def test_var_in_var(oso, t):
    oso.load_str("""
        allow("steve", "get", resource: Foo) if
            log in resource.logs and
            log.data = "hello";
    """)
    assert oso.is_allowed("steve", "get", fourth_foo)

    results = oso.authorized_resources("steve", "get", Foo)
    assert len(results) == 1


def test_parent_child_cases(oso, t):
    oso.load_str("""
        allow(log: Log, "thence", foo: Foo) if
          log.foo = foo;
        allow(log: Log, "thither", foo: Foo) if
          log in foo.logs;
        allow(log: Log, "glub", foo: Foo) if
          log.foo = foo and log in foo.logs;
        allow(log: Log, "bluh", foo: Foo) if
          log in foo.logs and log.foo = foo;
    """)
    foo = fourth_foo
    log = fourth_log_a
    check_authz(oso, log, "thence", Foo, [foo])
    check_authz(oso, log, "thither", Foo, [foo])
    check_authz(oso, log, "glub", Foo, [foo])
    check_authz(oso, log, "bluh", Foo, [foo])


def test_specializers(oso, t):
    oso.load_str("""
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
    """)
    logs = t['logs']
    foos = t['foos']
    parts = ["None", "Cls", "Dict", "Ptn"]
    for a in parts:
        for b in parts:
            for log in logs:
                for foo in filter(lambda f: f.id == log.foo_id, foos):
                    check_authz(oso, foo, a + b, Log, [log])


def test_ground_object_in_collection(oso, t):
    # value in var
    oso.load_str("""
        allow("steve", "get", resource: Foo) if
            1 in resource.numbers and 2 in resource.numbers;
    """)
    assert oso.is_allowed("steve", "get", fourth_foo)

    results = oso.authorized_resources("steve", "get", Foo)
    assert results == [fourth_foo]


@pytest.mark.xfail(reason="not yet supported")
def test_forall_in_collection(oso, t):
    oso.load_str('allow(_, _, bar: Bar) if forall(foo in bar.foos, foo.is_fooey = true);')
    results = oso.authorized_resources('gwen', 'get', t['Bar'])
    assert len(results) == 3

@pytest.mark.xfail(reason="not yet supported")
def test_no_objects_collection_condition(oso, t):
    oso.load_str('allow(_, _, bar: Bar) if not (foo in bar.foos and foo.is_fooey);')
    results = list(oso.authorized_resources('gwen', 'get', t['Bar']))
    assert len(results) == 0


def test_var_in_value(oso, t):
    # @TODO(steve): There is maybe a way to optimize the filter plan where if we are doing
    # two different of the same fetch with different fields we can combine them into an `in`.

    # var in value, This currently doesn't come through as an `in`
    # This is I think the thing that MikeD wants though, for this to come through
    # as an in so the SQL can do an IN.
    policy = """
    allow("steve", "get", resource: Log) if
        resource.data in ["hello", "world"];
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["fourth_log_a"])

    results = list(oso.authorized_resources("steve", "get", t["Log"]))
    assert unord_eq(results, [t["fourth_log_a"], t["third_log_b"]])


@pytest.mark.skip(
    """
    `or` constraints come from `not` negations and should instead be expanded in the
    simplifier"""
)
def test_or(oso, t):
    policy = """
    allow("steve", "get", r: Foo) if
        not (r.id = "something" and r.bar_id = "hello");
    """
    oso.load_str(policy)
    # assert oso.is_allowed("steve", "get", t['fourth_log_a'])

    results = list(oso.authorized_resources("steve", "get", t["Foo"]))
    assert len(results) == 2


def test_field_comparison(oso, t):
    policy = """
    allow("gwen", "eat", bar: Bar) if
        bar.is_cool = bar.is_still_cool;
    allow("gwen", "nom", bar: Bar) if
        bar.is_cool != bar.is_still_cool;
    """
    oso.load_str(policy)
    expected = [b for b in t["bars"] if b.is_cool == b.is_still_cool]
    check_authz(oso, "gwen", "eat", t["Bar"], expected)

    expected = [b for b in t["bars"] if b.is_cool != b.is_still_cool]
    check_authz(oso, "gwen", "nom", t["Bar"], expected)


@pytest.mark.xfail(reason="doesn't work yet!")
def test_field_cmp_rel_field(oso, t):
    policy = """
    allow(_, _, foo: Foo) if
        foo.bar.is_cool = foo.is_fooey;
    """
    oso.load_str(policy)
    expected = [t["another_foo"], t["third_foo"]]
    check_authz(oso, "gwen", "get", t["Foo"], expected)


def test_const_in_coll(oso, t):
    magic = 1
    oso.register_constant(magic, "magic")
    policy = """
    allow(_, _, foo: Foo) if
        magic in foo.numbers;
    """
    oso.load_str(policy)
    expected = [f for f in t["foos"] if magic in f.numbers]
    check_authz(oso, "gwen", "eat", t["Foo"], expected)


@pytest.mark.xfail(reason="negation unsupported")
def test_const_not_in_coll(oso, t):
    magic = 1
    oso.register_constant(magic, "magic")
    policy = """
    allow(_, _, foo: Foo) if
        not (magic in foo.numbers);
    """
    oso.load_str(policy)
    expected = [f for f in t["foos"] if magic not in f.numbers]
    check_authz(oso, "gwen", "eat", t["Foo"], expected)


def test_param_field(oso, t):
    policy = """
    allow(actor, action, resource: Log) if
        actor = resource.data and
        action = resource.id;
    """
    oso.load_str(policy)
    check_authz(oso, "steve", "c", t["Log"], [t["another_log_c"]])

