import pytest

from typing import Any, ClassVar
from dataclasses import dataclass
from oso import Oso, OsoError
from polar import Relationship


@pytest.fixture
def oso():
    oso = Oso()
    return oso


def filter_array(array, constraints):
    results = []
    for elem in array:
        matches = True
        for constraint in constraints:
            val = getattr(elem, constraint.field)
            if constraint.kind == "Eq":
                if val != constraint.value:
                    matches = False
                    break
            if constraint.kind == "In":
                if val not in constraint.value:
                    matches = False
                    break
            if constraint.kind == "Contains":
                if constraint.value not in val:
                    matches = False
                    break
        if matches:
            results.append(elem)
    return results


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
    something_foo = Foo(id="something", bar_id="hello", is_fooey=False, numbers=[])
    another_foo = Foo(id="another", bar_id="hello", is_fooey=True, numbers=[1])
    third_foo = Foo(id="third", bar_id="hello", is_fooey=True, numbers=[2])
    forth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True, numbers=[2, 1])

    forth_log_a = FooLogRecord(id="a", foo_id="fourth", data="hello")
    third_log_b = FooLogRecord(id="b", foo_id="third", data="world")
    another_log_c = FooLogRecord(id="c", foo_id="another", data="steve")

    bars = [hello_bar, goodbye_bar]
    foos = [something_foo, another_foo, third_foo, forth_foo]
    foo_logs = [forth_log_a, third_log_b, another_log_c]

    def get_bars(constraints):
        return filter_array(bars, constraints)

    def get_foos(constraints):
        return filter_array(foos, constraints)

    def get_foo_logs(constraints):
        return filter_array(foo_logs, constraints)

    oso.register_class(Bar, types={"id": str, "is_cool": bool}, fetcher=get_bars)
    oso.register_class(
        Foo,
        types={
            "id": str,
            "bar_id": str,
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
            # "bar": Relationship(
            #     kind="parent", other_type="Bar", my_field="bar_id", other_field="id"
            # ),
        },
        fetcher=get_foo_logs,
    )
    # Sorta hacky, just return anything you want to use in a test.
    return {
        "Foo": Foo,
        "FooLogRecord": FooLogRecord,
        "another_foo": another_foo,
        "forth_foo": forth_foo,
        "forth_log_a": forth_log_a,
    }


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
    assert oso.is_allowed("steve", "get", t["forth_foo"])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 1


def test_val_in_var(oso, t):
    # value in var
    oso.clear_rules()
    policy = """
    allow("steve", "get", resource: Foo) if
        1 in resource.numbers and 2 in resource.numbers;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", t["forth_foo"])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 1


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
    assert oso.is_allowed("steve", "get", t["forth_log_a"])

    results = list(oso.get_allowed_resources("steve", "get", t["FooLogRecord"]))
    assert len(results) == 2


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
    # assert oso.is_allowed("steve", "get", t['forth_log_a'])

    results = list(oso.get_allowed_resources("steve", "get", t["Foo"]))
    assert len(results) == 2


def test_roles_data_filtering(oso):
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

    ios = Repo(name="ios", org_name="apple")
    oso_repo = Repo(name="oso", org_name="osohq")
    demo_repo = Repo(name="demo", org_name="osohq")

    ios_laggy = Issue(name="laggy", repo_name="ios")
    oso_bug = Issue(name="bug", repo_name="oso")

    leina = User(name="leina")
    steve = User(name="steve")
    gabe = User(name="gabe")

    users = [leina, steve, gabe]
    orgs = [apple, osohq]
    repos = [ios, oso_repo, demo_repo]
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

    parent_child(parent_org, repo: Repo) if
        repo.org = parent_org and
        parent_org matches Org;

    parent_child(parent_repo, issue: Issue) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_has_role_for_resource(actor, role_name: String, resource) if
        role in actor.roles and
        role.resource_name = resource.name and
        role.role = role_name;

    allow(actor, action, resource) if
        role_allows(actor, action, resource);
    """

    oso.load_str(policy)
    oso.enable_roles()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "create_repo", osohq)
    assert oso.is_allowed(leina, "push", oso_repo)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "edit", oso_bug)

    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "create_repo", osohq)
    assert not oso.is_allowed(steve, "push", oso_repo)
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    assert not oso.is_allowed(leina, "edit", ios_laggy)
    assert not oso.is_allowed(steve, "edit", ios_laggy)

    results = list(oso.get_allowed_resources(leina, "pull", Repo))
    assert len(results) == 2

    # TODO(steve): infinite loop!
    # results = list(oso.get_allowed_resources(leina, "edit", Issue))
    # assert results == [oso_bug]

    results = list(oso.get_allowed_resources(leina, "invite", Org))
    assert results == [osohq]
