from dataclasses import dataclass
from typing import Dict


@dataclass
class User:
    name: str
    org_roles: [Dict[str, str]]


@dataclass
class Organization:
    name: str


@dataclass
class Repository:
    name: str
    org: Organization


from oso import Oso, Relation

steve = User(name="steve", org_roles=[{"role": "owner", "org_name": "osohq"}])
damien = User(name="damien", org_roles=[{"role": "member", "org_name": "apple"}])

apple = Organization(name="apple")
osohq = Organization(name="osohq")

ios_repo = Repository(name="ios", org=apple)
oso_repo = Repository(name="oso", org=osohq)
demo_repo = Repository(name="demo", org=osohq)

oso = Oso()
oso.register_class(User)
oso.register_class(Organization)
oso.register_class(Repository)


oso.load_str(
    """
    allow(actor, action, resource) if
        has_permission(actor, action, resource);
    has_role(user: User, name: String, org: Organization) if
        role in user.org_roles and
        role.role = name and
        role.org_name = org.name;
    actor User {}
    resource Organization {
        roles = ["owner", "member"];
        permissions = ["invite", "create_repo"];
        "create_repo" if "member";
        "invite" if "owner";
        "member" if "owner";
    }
    resource Repository {
        roles = [ "writer", "reader" ];
        permissions = [ "push", "pull" ];
        relations = { parent: Organization };
        
        "pull" if "reader";
        "push" if "writer";
        
        "reader" if "writer";
        
        "reader" if "member" on "parent";
        "writer" if "owner" on "parent";
      }
      has_relation(org: Organization, "parent", repo: Repository) if
        org = repo.org;
    """
)

result = list(oso.query_rule("allow", steve, "invite", osohq))
# assert oso.is_allowed(steve, "push", oso_repo)
