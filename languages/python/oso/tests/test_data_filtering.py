import pytest

from sqlalchemy import create_engine
from sqlalchemy.types import String, Boolean
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from dataclasses import dataclass
from oso import Oso
from polar import Relationship
from functools import reduce


@pytest.fixture
def oso():
    oso = Oso()
    return oso


def fold_constraints(constraints):
    return reduce(
        lambda f, g: lambda x: f(x) and g(x),
        [c.check for c in constraints],
        lambda _: True,
    )


def filter_array(array, constraints):
    check = fold_constraints(constraints)
    return [x for x in array if check(x)]


def unord_eq(a, b):
    for x in a:
        try:
            b.remove(x)
        except ValueError:
            return False
    return not b


# Shared test setup.
@pytest.fixture
def t(oso):
    # Register some types and callbacks
    @dataclass
    class Bar:
        id: str
        is_cool: bool
        is_still_cool: bool

    @dataclass
    class Foo:
        id: str
        bar_id: str
        is_fooey: bool
        numbers: list

    @dataclass
    class FooLogRecord:
        id: str
        foo_id: str
        data: str

    hello_bar = Bar(id="hello", is_cool=True, is_still_cool=True)
    goodbye_bar = Bar(id="goodbye", is_cool=False, is_still_cool=True)
    hershey_bar = Bar(id="hershey", is_cool=False, is_still_cool=False)
    something_foo = Foo(id="something", bar_id="hello", is_fooey=False, numbers=[])
    another_foo = Foo(id="another", bar_id="hello", is_fooey=True, numbers=[1])
    third_foo = Foo(id="third", bar_id="hello", is_fooey=True, numbers=[2])
    fourth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True, numbers=[2, 1])

    fourth_log_a = FooLogRecord(id="a", foo_id="fourth", data="hello")
    third_log_b = FooLogRecord(id="b", foo_id="third", data="world")
    another_log_c = FooLogRecord(id="c", foo_id="another", data="steve")

    bars = [hello_bar, goodbye_bar, hershey_bar]
    foos = [something_foo, another_foo, third_foo, fourth_foo]
    foo_logs = [fourth_log_a, third_log_b, another_log_c]

    def get_bars(constraints):
        return filter_array(bars, constraints)

    def get_foos(constraints):
        return filter_array(foos, constraints)

    def get_foo_logs(constraints):
        return filter_array(foo_logs, constraints)

    oso.register_class(
        Bar, types={"id": str, "is_cool": bool, "is_still_cool": bool}, fetcher=get_bars
    )
    oso.register_class(
        Foo,
        types={
            "id": str,
            "bar_id": str,
            "is_fooey": bool,
            "numbers": list,
            "bar": Relationship(
                kind="parent", other_type="Bar", my_field="bar_id", other_field="id"
            ),
            "logs": Relationship(
                kind="children",
                other_type="FooLogRecord",
                my_field="id",
                other_field="foo_id",
            ),
        },
        fetcher=get_foos,
    )
    oso.register_class(
        FooLogRecord,
        types={
            "id": str,
            "foo_id": str,
            "data": str,
            "foo": Relationship(
                kind="parent", other_type="Foo", my_field="foo_id", other_field="id"
            ),
        },
        fetcher=get_foo_logs,
    )
    # Sorta hacky, just return anything you want to use in a test.
    return {
        "Foo": Foo,
        "Bar": Bar,
        "FooLogRecord": FooLogRecord,
        "another_foo": another_foo,
        "third_foo": third_foo,
        "fourth_foo": fourth_foo,
        "fourth_log_a": fourth_log_a,
        "third_log_b": third_log_b,
        "another_log_c": another_log_c,
        "bars": bars,
        "foos": foos,
        "logs": foo_logs,
    }


# Shared test setup.
@pytest.fixture
def sqlalchemy_t(oso):
    Base = declarative_base()

    class Bar(Base):  # type: ignore
        __tablename__ = "bars"

        id = Column(String(), primary_key=True)
        is_cool = Column(Boolean())
        is_still_cool = Column(Boolean())

    class Foo(Base):  # type: ignore
        __tablename__ = "foos"

        id = Column(String(), primary_key=True)
        bar_id = Column(String, ForeignKey("bars.id"))
        is_fooey = Column(Boolean())

    engine = create_engine("sqlite:///:memory:")

    Session = sessionmaker(bind=engine)
    session = Session()

    Base.metadata.create_all(engine)

    # @TODO: Somehow the session needs to get in here, didn't think about that yet... Just hack for now and use a global
    # one.
    def get_bars(constraints):
        query = session.query(Bar)
        for constraint in constraints:
            field = getattr(Bar, constraint.field)
            if constraint.kind == "Eq":
                query = query.filter(field == constraint.value)
            elif constraint.kind == "In":
                query = query.filter(field.in_(constraint.value))
            # ...
        return query.all()

    oso.register_class(
        Bar, types={"id": str, "is_cool": bool, "is_still_cool": bool}, fetcher=get_bars
    )

    def get_foos(constraints):
        query = session.query(Foo)
        for constraint in constraints:
            field = getattr(Foo, constraint.field)
            if constraint.kind == "Eq":
                query = query.filter(field == constraint.value)
            elif constraint.kind == "In":
                query = query.filter(field.in_(constraint.value))
            # ...
        return query.all()

    oso.register_class(
        Foo,
        types={
            "id": str,
            "bar_id": str,
            "is_fooey": bool,
            "bar": Relationship(
                kind="parent", other_type="Bar", my_field="bar_id", other_field="id"
            ),
        },
        fetcher=get_foos,
    )

    hello_bar = Bar(id="hello", is_cool=True, is_still_cool=True)
    goodbye_bar = Bar(id="goodbye", is_cool=False, is_still_cool=True)
    hershey_bar = Bar(id="hershey", is_cool=False, is_still_cool=False)
    something_foo = Foo(id="something", bar_id="hello", is_fooey=False)
    another_foo = Foo(id="another", bar_id="hello", is_fooey=True)
    third_foo = Foo(id="third", bar_id="hello", is_fooey=True)
    fourth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True)

    for obj in [
        hello_bar,
        goodbye_bar,
        hershey_bar,
        something_foo,
        another_foo,
        third_foo,
        fourth_foo,
    ]:
        session.add(obj)
        session.commit()

    return {"session": Session, "Bar": Bar, "Foo": Foo, "another_foo": another_foo}


def test_sqlalchemy_relationship(oso, sqlalchemy_t):
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.bar = bar and
        bar.is_cool = true and
        resource.is_fooey = true;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", sqlalchemy_t["another_foo"])

    results = list(oso.get_allowed_resources("steve", "get", sqlalchemy_t["Foo"]))
    assert len(results) == 2


def test_no_relationships(oso, t):
    # Write a policy
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.is_fooey = true;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["another_foo"])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 3


def test_relationship(oso, t):
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.bar = bar and
        bar.is_cool = true and
        resource.is_fooey = true;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["another_foo"])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 2


def test_known_results(oso):
    policy = """
      allow(_, _, i: Integer) if i in [1, 2];
      allow(_, _, d: Dictionary) if d = {};
    """
    oso.load_str(policy)

    results = oso.get_allowed_resources("gwen", "get", int)
    assert unord_eq(results, [1, 2])

    results = oso.get_allowed_resources("gwen", "get", dict)
    assert results == [{}]

    results = oso.get_allowed_resources("gwen", "get", str)
    assert results == []


def test_var_in_values(oso, t):
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.bar = bar and
        bar.is_cool in [true, false];
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["another_foo"])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 4


def test_var_in_var(oso, t):
    policy = """
    allow("steve", "get", resource: Foo) if
        log in resource.logs and
        log.data = "hello";
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["fourth_foo"])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 1


def test_parent_child(oso, t):
    policy = """
    allow(log: FooLogRecord, "thence", foo: Foo) if
      log.foo = foo;
    allow(log: FooLogRecord, "thither", foo: Foo) if
      log in foo.logs;
    allow(log: FooLogRecord, "glub", foo: Foo) if
      log.foo = foo and log in foo.logs;
    allow(log: FooLogRecord, "bluh", foo: Foo) if
      log in foo.logs and log.foo = foo;
    """
    oso.load_str(policy)
    foo = t["fourth_foo"]
    log = t["logs"][0]
    check_authz(oso, log, "thence", t["Foo"], [foo])
    check_authz(oso, log, "thither", t["Foo"], [foo])
    check_authz(oso, log, "glub", t["Foo"], [foo])
    # check_authz(oso, log, "bluh", t["Foo"], [foo]) # FIXME stack overflow :(


def test_val_in_var(oso, t):
    # value in var
    oso.clear_rules()
    policy = """
    allow("steve", "get", resource: Foo) if
        1 in resource.numbers and 2 in resource.numbers;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["fourth_foo"])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert results == [t["fourth_foo"]]


def test_var_in_value(oso, t):
    # @TODO(steve): There is maybe a way to optimize the filter plan where if we are doing
    # two different of the same fetch with different fields we can combine them into an `in`.

    # var in value, This currently doesn't come through as an `in`
    # This is I think the thing that MikeD wants though, for this to come through
    # as an in so the SQL can do an IN.
    policy = """
    allow("steve", "get", resource: FooLogRecord) if
        resource.data in ["hello", "world"];
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["fourth_log_a"])

    results = list(oso.get_allowed_resources("steve", "get", t["FooLogRecord"]))
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

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 2


def test_field_cmp_field(oso, t):
    policy = """
    allow(_, _, bar: Bar) if
        bar.is_cool = bar.is_still_cool;
    """
    oso.load_str(policy)
    expected = [b for b in t["bars"] if b.is_cool == b.is_still_cool]
    check_authz(oso, "gwen", "eat", t["Bar"], expected)


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
    allow(actor, action, resource: FooLogRecord) if
        actor = resource.data and
        action = resource.id;
    """
    oso.load_str(policy)
    check_authz(oso, "steve", "c", t["FooLogRecord"], [t["another_log_c"]])


@pytest.fixture
def roles(oso):
    # Register some types and callbacks
    @dataclass
    class User:
        name: str

    @dataclass
    class Org:
        name: str

    @dataclass
    class Repo:
        name: str
        org_name: str

    @dataclass
    class Issue:
        name: str
        repo_name: str

    @dataclass
    class Role:
        user_name: str
        resource_name: str
        role: str

    apple = Org(name="apple")
    osohq = Org(name="osohq")

    ios_repo = Repo(name="ios", org_name="apple")
    oso_repo = Repo(name="oso", org_name="osohq")
    demo_repo = Repo(name="demo", org_name="osohq")

    ios_laggy = Issue(name="laggy", repo_name="ios")
    oso_bug = Issue(name="bug", repo_name="oso")

    leina = User(name="leina")
    steve = User(name="steve")
    gabe = User(name="gabe")

    users = [leina, steve, gabe]
    orgs = [apple, osohq]
    repos = [ios_repo, oso_repo, demo_repo]
    issues = [ios_laggy, oso_bug]

    roles = [
        Role(user_name="leina", resource_name="osohq", role="owner"),
        Role(user_name="steve", resource_name="osohq", role="member"),
        Role(user_name="gabe", resource_name="oso", role="writer"),
    ]

    def get_orgs(constraints):
        return filter_array(orgs, constraints)

    def get_repos(constraints):
        return filter_array(repos, constraints)

    def get_issues(constraints):
        return filter_array(issues, constraints)

    def get_roles(constraints):
        return filter_array(roles, constraints)

    def get_users(constraints):
        return filter_array(users, constraints)

    oso.register_class(Org, types={"name": str}, fetcher=get_orgs)
    oso.register_class(
        Repo,
        types={
            "name": str,
            "org_name": str,
            "org": Relationship(
                kind="parent", other_type="Org", my_field="org_name", other_field="name"
            ),
        },
        fetcher=get_repos,
    )
    oso.register_class(
        Issue,
        types={
            "name": str,
            "repo_name": str,
            "repo": Relationship(
                kind="parent",
                other_type="Repo",
                my_field="repo_name",
                other_field="name",
            ),
        },
        fetcher=get_issues,
    )
    oso.register_class(
        Role,
        types={
            "user_name": str,
            "resource_name": str,
            "role": str,
        },
        fetcher=get_roles,
    )
    oso.register_class(
        User,
        types={
            "name": str,
            "roles": Relationship(
                kind="children",
                other_type="Role",
                my_field="name",
                other_field="user_name",
            ),
        },
        fetcher=get_users,
    )

    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            member: {
                permissions: ["create_repo"],
                implies: ["repo:reader"]
            },
            owner: {
                permissions: ["invite"],
                implies: ["repo:writer", "member"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            writer: {
                permissions: ["push", "issue:edit"],
                implies: ["reader"]
            },
            reader: {
                permissions: ["pull"]
            }
        };

    resource(_type: Issue, "issue", actions, {}) if
        actions = [
            "edit"
        ];

    parent_child(parent_org: Org, repo: Repo) if
        repo.org = parent_org;

    parent_child(parent_repo: Repo, issue: Issue) if
        issue.repo = parent_repo;

    actor_has_role_for_resource(actor, role_name: String, resource) if
        role in actor.roles and
        role.resource_name = resource.name and
        role.role = role_name;

    allow(actor, action, resource) if
        role_allows(actor, action, resource);
    """

    oso.load_str(policy)
    oso.enable_roles()
    return {
        "apple": apple,
        "osohq": osohq,
        "steve": steve,
        "leina": leina,
        "gabe": gabe,
        "oso": oso_repo,
        "ios": ios_repo,
        "demo": demo_repo,
        "bug": oso_bug,
        "laggy": ios_laggy,
        "Role": Role,
        "Repo": Repo,
        "Issue": Issue,
        "Org": Org,
        "User": User,
    }


def check_authz(oso, actor, action, resource, expected):
    assert unord_eq(oso.get_allowed_resources(actor, action, resource), expected)
    for re in expected:
        assert oso.is_allowed(actor, action, re)


def test_roles_data_filtering_owner(oso, roles):
    leina = roles["leina"]
    osohq = roles["osohq"]
    oso_repo = roles["oso"]
    demo_repo = roles["demo"]
    oso_bug = roles["bug"]
    Org = roles["Org"]
    Repo = roles["Repo"]
    Issue = roles["Issue"]

    check_authz(oso, leina, "invite", Org, [osohq])
    check_authz(oso, leina, "pull", Repo, [oso_repo, demo_repo])
    check_authz(oso, leina, "push", Repo, [oso_repo, demo_repo])
    check_authz(oso, leina, "edit", Issue, [oso_bug])


def test_roles_data_filtering_member(oso, roles):
    steve = roles["steve"]
    oso_repo = roles["oso"]
    demo_repo = roles["demo"]
    Repo = roles["Repo"]
    Issue = roles["Issue"]
    Org = roles["Org"]

    check_authz(oso, steve, "pull", Repo, [oso_repo, demo_repo])
    check_authz(oso, steve, "push", Repo, [])
    check_authz(oso, steve, "invite", Org, [])
    check_authz(oso, steve, "edit", Issue, [])


def test_roles_data_filtering_writer(oso, roles):
    gabe = roles["gabe"]
    Issue = roles["Issue"]
    Org = roles["Org"]
    Repo = roles["Repo"]
    oso_bug = roles["bug"]
    oso_repo = roles["oso"]

    check_authz(oso, gabe, "invite", Org, [])
    check_authz(oso, gabe, "pull", Repo, [oso_repo])
    check_authz(oso, gabe, "push", Repo, [oso_repo])
    check_authz(oso, gabe, "edit", Issue, [oso_bug])
