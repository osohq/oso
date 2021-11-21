import pytest
from oso import Relation, ArrayAdapter
from .helpers import filter_array, DfTestOso
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
def oso():
    oso = DfTestOso()

    type_arrays = {
        Org: orgs,
        Repo: repos,
        Issue: issues,
        Role: roles,
        User: users
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

def test_it(oso):
    query = oso.authzd_query(leina, "edit", Issue)
    print(query)
