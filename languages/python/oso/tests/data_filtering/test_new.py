import itertools
from typing import List

import pytest
from helpers import DfTestOso, unord_eq
from sqlalchemy import create_engine, not_
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.types import Boolean, String

from oso import Relation
from polar.data.adapter.sqlalchemy_adapter import SqlAlchemyAdapter

Base = declarative_base()


class Bar(Base):  # type: ignore
    __tablename__ = "bars"

    id = Column(String(), primary_key=True)
    is_cool = Column(Boolean())
    is_still_cool = Column(Boolean())

    def foos(self):
        return [foo for foo in foos if foo.bar_id == self.id]


class Foo(Base):  # type: ignore
    __tablename__ = "foos"

    id = Column(String(), primary_key=True)
    bar_id = Column(String(), ForeignKey("bars.id"))
    is_fooey = Column(Boolean())

    def bar(self):
        one_bar = [bar for bar in bars if bar.id == self.bar_id]
        assert len(one_bar) == 1
        return one_bar[0]

    def logs(self):
        return [log for log in logs if self.id == log.foo_id]


class Log(Base):  # type: ignore
    __tablename__ = "logs"

    id = Column(String(), primary_key=True)
    foo_id = Column(String(), ForeignKey("foos.id"))
    data = Column(String())

    def foo(self):
        one_foo = [foo for foo in foos if foo.id == self.foo_id]
        assert len(one_foo) == 1
        return one_foo[0]


engine = create_engine("sqlite:///:memory:")
Base.metadata.create_all(engine)
Session = sessionmaker(bind=engine)
session = Session()

hello_bar = Bar(id="hello", is_cool=True, is_still_cool=True)
goodbye_bar = Bar(id="goodbye", is_cool=False, is_still_cool=True)
hershey_bar = Bar(id="hershey", is_cool=False, is_still_cool=False)

something_foo = Foo(id="something", bar_id="hello", is_fooey=False)
another_foo = Foo(id="another", bar_id="hello", is_fooey=True)
third_foo = Foo(id="third", bar_id="hello", is_fooey=True)
fourth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True)

fourth_log_a = Log(id="a", foo_id="fourth", data="hello")
third_log_b = Log(id="b", foo_id="third", data="world")
another_log_c = Log(id="c", foo_id="another", data="steve")

bars = [hello_bar, goodbye_bar, hershey_bar]
foos = [something_foo, another_foo, third_foo, fourth_foo]
logs = [fourth_log_a, third_log_b, another_log_c]

colls: List[list] = [bars, foos, logs]
for coll in colls:
    for obj in coll:
        session.add(obj)
        session.commit()

binary_predicates = {
    "Eq": lambda a, b: a == b,
    "Neq": lambda a, b: a != b,
    "In": lambda a, b: a.in_(b),
    "Nin": lambda a, b: not_(a.in_(b)),
}


@pytest.fixture
def oso():
    oso = DfTestOso()
    oso.set_data_filtering_adapter(SqlAlchemyAdapter(session))

    # @TODO: Somehow the session needs to get in here, didn't think about that yet... Just hack for now and use a global
    # one.
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
    )

    oso.register_class(
        Log,
        fields={
            "id": str,
            "data": str,
            "foo": Relation(
                kind="one", other_type="Foo", my_field="foo_id", other_field="id"
            ),
        },
    )

    oso.register_class(
        Foo,
        fields={
            "id": str,
            "bar_id": str,
            "is_fooey": bool,
            "bar": Relation(
                kind="one", other_type="Bar", my_field="bar_id", other_field="id"
            ),
            "logs": Relation(
                kind="many", other_type="Log", my_field="id", other_field="foo_id"
            ),
        },
    )

    return oso


# cf. test_flask_model
def test_model(oso):
    oso.load_str('allow("gwen", "get", foo: Foo) if foo.id = "something";')
    oso.check_authz("gwen", "get", Foo, [something_foo])

    oso.clear_rules()
    oso.load_str(
        """
            allow("gwen", "get", foo: Foo) if foo.id = "something";
            allow("gwen", "get", foo: Foo) if foo.id = "another";
        """
    )
    oso.check_authz("gwen", "get", Foo, [another_foo, something_foo])


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
        oso.check_authz(bar, "read", Foo, expected)


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
            if (foo.bar().is_cool and foo.bar() is bar)
            or (foo.bar().is_cool and foo.is_fooey)
            or (not foo.bar().is_cool and bar.is_still_cool)
        ]
        oso.check_authz(bar, "read", Foo, expected)


def test_nested_relationship_many_single(oso):
    oso.load_str(
        """
        allow(log: Log, "read", bar: Bar) if
            log.foo in bar.foos;
    """
    )
    for log in logs:
        expected = [bar for bar in bars if log.foo() in bar.foos()]
        oso.check_authz(log, "read", Bar, expected)


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
        oso.check_authz(log, "read", Bar, expected)


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
        oso.check_authz(log, "read", Bar, expected)


def test_partial_in_collection(oso):
    oso.load_str('allow(bar, "read", foo: Foo) if foo in bar.foos;')
    for bar in bars:
        oso.check_authz(bar, "read", Foo, bar.foos())


def test_empty_constraints_in(oso):
    oso.load_str('allow(_, "read", foo: Foo) if _ in foo.logs;')
    expected = [foo for foo in foos if foo.logs()]
    oso.check_authz("gwen", "read", Foo, expected)


def test_in_with_constraints_but_no_matching_object(oso):
    oso.load_str('allow(_, "read", foo: Foo) if log in foo.logs and log.data = "nope";')
    oso.check_authz("gwen", "read", Foo, [])


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

    expected = [bar for bar in bars if bar.foos()]
    assert unord_eq(expected, [hello_bar, goodbye_bar])
    oso.check_authz("gwen", "read", Bar, expected)


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


def test_partial_isa_with_path(oso):
    oso.load_str(
        """
            allow(_, _, foo: Foo) if check(foo.bar);
            check(bar: Bar) if bar.id = "goodbye";   # this should match
            check(foo: Foo) if foo.bar.id = "hello"; # this shouldn't match
        """
    )
    oso.check_authz("gwen", "read", Foo, goodbye_bar.foos())


def test_no_relationships(oso):
    oso.load_str('allow("steve", "get", foo: Foo) if foo.is_fooey = true;')
    expected = [foo for foo in foos if foo.is_fooey]
    oso.check_authz("steve", "get", Foo, expected)


def test_neq(oso):
    oso.load_str('allow("steve", action, foo: Foo) if foo.bar.id != action;')

    for bar in bars:
        expected = [foo for foo in foos if foo.bar() != bar]
        oso.check_authz("steve", bar.id, Foo, expected)


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
    oso.check_authz("steve", "get", Foo, expected)


@pytest.mark.xfail(reason="not supported yet")
def test_duplex_relationship(oso):
    oso.load_str("allow(_, _, foo: Foo) if foo in foo.bar.foos;")
    oso.check_authz("gwen", "gwen", Foo, foos)


def test_scalar_in_list(oso):
    oso.load_str(
        """
            allow("steve", "get", resource: Foo) if
                resource.bar = bar and
                bar.is_cool in [true, false];
        """
    )
    oso.check_authz("steve", "get", Foo, foos)


def test_var_in_var(oso):
    oso.load_str(
        """
            allow("steve", "get", foo: Foo) if
                log in foo.logs and
                log.data = "hello";
        """
    )
    expected = [foo for foo in foos for log in foo.logs() if log.data == "hello"]
    assert fourth_foo in expected
    oso.check_authz("steve", "get", Foo, expected)


def test_parent_child_cases(oso):
    policy = """
        allow(_: Log{foo: foo}, 0, foo: Foo);
        allow(log: Log, 1, _: Foo{logs: logs}) if log in logs;
        allow(log: Log{foo: foo}, 2, foo: Foo{logs: logs}) if log in logs;
    """
    oso.load_str(policy)
    for action, log in itertools.product(range(3), logs):
        oso.check_authz(log, action, Foo, [log.foo()])


def test_specializers(oso):
    policy = """
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
    oso.load_str(policy)
    parts = ["None", "Cls", "Dict", "Ptn"]
    for a in parts:
        for b, log in itertools.product(parts, logs):
            oso.check_authz(log.foo(), a + b, Log, [log])


def test_var_in_value(oso):
    # @TODO(steve): There is maybe a way to optimize the filter plan where if we are doing
    # two different of the same fetch with different fields we can combine them into an `in`.

    # var in value, This currently doesn't come through as an `in`
    # This is I think the thing that MikeD wants though, for this to come through
    # as an in so the SQL can do an IN.
    oso.load_str('allow(_, _, log: Log) if log.data in ["hello", "world"];')
    oso.check_authz("steve", "get", Log, [third_log_b, fourth_log_a])


def test_field_eq(oso):
    oso.load_str("allow(_, _, _: Bar{is_cool: cool, is_still_cool: cool});")
    expected = [b for b in bars if b.is_cool == b.is_still_cool]
    oso.check_authz("gwen", "get", Bar, expected)


def test_field_neq(oso):
    oso.load_str("allow(_, _, bar: Bar) if bar.is_cool != bar.is_still_cool;")
    expected = [b for b in bars if b.is_cool != b.is_still_cool]
    oso.check_authz("gwen", "get", Bar, expected)


def test_param_field(oso):
    oso.load_str("allow(data, id, _: Log{data: data, id: id});")
    for log in logs:
        oso.check_authz(log.data, log.id, Log, [log])


def test_field_cmp_rel_field(oso):
    oso.load_str("allow(_, _, foo: Foo) if foo.bar.is_cool = foo.is_fooey;")
    expected = [foo for foo in foos if foo.is_fooey == foo.bar().is_cool]
    oso.check_authz("gwen", "get", Foo, expected)


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


@pytest.mark.xfail(reason="not yet supported")
def test_forall_in_collection(oso):
    oso.load_str(
        "allow(_, _, bar: Bar) if forall(foo in bar.foos, foo.is_fooey = true);"
    )
    results = oso.authorized_resources("", "get", Bar)
    assert len(results) == 3


@pytest.mark.xfail(reason="not yet supported")
def test_no_objects_collection_condition(oso):
    oso.load_str("allow(_, _, bar: Bar) if not (foo in bar.foos and foo.is_fooey);")
    results = oso.authorized_resources("", "get", Bar)
    assert len(results) == 0


@pytest.mark.xfail(reason="a bug")
def test_unify_ins_neq(oso):
    # can read any bar with at least two foos
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
    oso.check_authz("", "read", Bar, expected)


@pytest.mark.xfail(reason="a bug")
def test_deeply_nested_in(oso):
    # can read any foo whose bar has another foo.
    oso.load_str(
        """
            allow(_, "read", a: Foo) if
                b in a.bar.foos and b != a and
                c in b.bar.foos and c != b and
                d in c.bar.foos and d != c and
                e in d.bar.foos and e != d;
        """
    )

    result = oso.authorized_resources("", "read", Foo)
    assert len(result) == 3


def test_two_level_isa_with_path(oso):
    oso.load_str(
        """
        allow(u, _, log: Log) if check(u, log.foo.bar);
        check(u, log: Log) if allow(u, "", log);
        check(_u, _f: Foo);
        check(_, _: Bar);
    """
    )

    result = oso.authorized_resources("", "read", Log)
    assert len(result) == 3
