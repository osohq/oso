import pytest
from dataclasses import dataclass
from oso import Relation
from helpers import *

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

@pytest.fixture
def roles(oso):
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

    def exec_query(results):
        return results

    def combine_query(q1, q2):
        results = q1 + q2
        return [i for n, i in enumerate(results) if i not in results[:n]]

    oso.register_class(
        Org,
        fields={"name": str},
        build_query=get_orgs,
        exec_query=exec_query,
        combine_query=combine_query,
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
        build_query=get_repos,
        exec_query=exec_query,
        combine_query=combine_query,
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
        build_query=get_issues,
        exec_query=exec_query,
        combine_query=combine_query,
    )
    oso.register_class(
        Role,
        fields={
            "user_name": str,
            "resource_name": str,
            "role": str,
        },
        build_query=get_roles,
        exec_query=exec_query,
        combine_query=combine_query,
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
        build_query=get_users,
        exec_query=exec_query,
        combine_query=combine_query,
    )

    policy = """
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

    oso.load_str(policy)

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


