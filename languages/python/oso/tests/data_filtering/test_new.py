import pytest
from oso import Relation, ArrayAdapter
from helpers import filter_array, DfTestOso
from dataclasses import dataclass


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


# Register some types and callbacks
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


@pytest.fixture
def oso_roles():
    oso = DfTestOso()

    type_arrays = {
        'Org': orgs,
        'Repo': repos,
        'Issue': issues,
        'Role': roles,
        'User': users
    }
    adapter = ArrayAdapter(type_arrays=type_arrays)
    oso.set_data_filtering_adapter(adapter)

    oso.register_class(
        Org,
        fields={"name": str},
    )
    oso.register_class(
        Repo,
        fields={
            "name": str,
            "org_name": str,
            "org": Relation(
                kind="one", other_type="Org", my_field="org_name", other_field="name"
            ),
        },
    )
    oso.register_class(
        Issue,
        fields={
            "name": str,
            "repo_name": str,
            "repo": Relation(
                kind="one",
                other_type="Repo",
                my_field="repo_name",
                other_field="name",
            ),
        },
    )
    oso.register_class(
        Role,
        fields={
            "user_name": str,
            "resource_name": str,
            "role": str,
        },
    )
    oso.register_class(
        User,
        fields={
            "name": str,
            "roles": Relation(
                kind="many",
                other_type="Role",
                my_field="name",
                other_field="user_name",
            ),
        },
    )

    oso.load_str(
        """
        allow(actor, action, resource) if
          has_permission(actor, action, resource);

        has_role(user: User, name: String, resource: Resource) if
          role in user.roles and
          role.role = name and
          role.resource_name = resource.name;

        actor User {}

        resource Org {
          roles = [ "owner", "member" ];
          permissions = [ "invite", "create_repo" ];

          "create_repo" if "member";
          "invite" if "owner";

          "member" if "owner";
        }

        resource Repo {
          roles = [ "writer", "reader" ];
          permissions = [ "push", "pull" ];
          relations = { parent: Org };

          "pull" if "reader";
          "push" if "writer";

          "reader" if "writer";

          "reader" if "member" on "parent";
          "writer" if "owner" on "parent";
        }

        has_relation(org: Org, "parent", repo: Repo) if
          org = repo.org;

        resource Issue {
          permissions = [ "edit" ];
          relations = { parent: Repo };

          "edit" if "writer" on "parent";
        }

        has_relation(repo: Repo, "parent", issue: Issue) if
          repo = issue.repo;
    """
    )

    return oso

def test_roles(oso_roles):
    query = oso_roles.authorized_query(leina, "edit", Issue)
    print(query)


@dataclass
class Bar:
    id: str
    is_cool: bool
    is_still_cool: bool

    def foos(self):
        return [foo for foo in foos if foo.bar_id == self.id]

    def __hash__(self) -> int:
        return hash(self.id)


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

    def __hash__(self) -> int:
        return hash(self.id)


@dataclass
class Log:
    id: str
    foo_id: str
    data: str

    def foo(self):
        one_foo = [foo for foo in foos if foo.id == self.foo_id]
        assert len(one_foo) == 1
        return one_foo[0]

    def __hash__(self) -> int:
        return hash(self.id)


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
def oso_foo():
    oso = DfTestOso()

    type_arrays = {
        'Foo': foos,
        'Bar': bars,
        'Log': logs
    }
    adapter = ArrayAdapter(type_arrays=type_arrays)
    oso.set_data_filtering_adapter(adapter)

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
    )
    return oso

def test_foo(oso_foo):
    oso_foo.load_str('allow(_, _, _: Foo{id: "something"});')
    oso_foo.check_authz("gwen", "get", Foo, [something_foo])

    oso_foo.clear_rules()
    oso_foo.load_str(
        """
            allow(_, _, _: Foo{id: "something"});
            allow(_, _, _: Foo{id: "another"});
        """
    )
    oso_foo.check_authz("gwen", "get", Foo, [another_foo, something_foo])