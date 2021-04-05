import pytest

from oso import Oso, polar_class, OsoRoles
from polar import exceptions


class User:
    name: str = ""

    def __init__(self, name=""):
        self.name = name

class Organization:
    id: str = ""
    def __init__(self, id):
        self.id = id

class Repository:
    id: str = ""
    public: bool
    org: Organization

    def __init__(self, id, org, public=False):
        self.id = id
        self.public = public
        self.org = org

class Issue:
    id: str = ""
    public: bool
    repo: Repository

    def __init__(self, id, repo, public=False):
        self.id = id
        self.public = public
        self.repo = repo

def test_roles():
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Repository)
    oso.register_class(Organization)
    oso.register_class(Issue)

    roles = OsoRoles()

    # Repo permissions
    permission_repository_read = roles.new_permission(resource=Repository, action="read")
    permission_repository_write = roles.new_permission(resource=Repository, action="write")
    permission_repository_list_issues = roles.new_permission(resource=Repository, action="list_issues")

    # Repository roles
    role_repository_read = roles.new_role(resource=Repository, name="READ")
    role_repository_write = roles.new_role(resource=Repository, name="WRITE")
    role_repository_admin = roles.new_role(resource=Repository, name="ADMIN")

    # Issue permissions
    permission_issue_read = roles.new_permission(resource=Issue, action="read")
    permission_issue_write = roles.new_permission(resource=Issue, action="write")

    # Issue-repo relationship
    roles.new_relationship(name="issue_repo", child=Issue, parent=Repository, get=lambda child: child.repo)

    # Organization roles
    role_organization_owner = roles.new_role(resource=Organization, name="OWNER")

    # Repo-org relationship
    roles.new_relationship(name="repo_org", child=Repository, parent=Organization, get=lambda child: child.org)

    # Permission assignment
    roles.new_role_permission(role=role_repository_read, permission=permission_repository_read)
    roles.new_role_permission(role=role_repository_read, permission=permission_repository_list_issues)
    roles.new_role_permission(role=role_repository_read, permission=permission_issue_read)

    roles.new_role_permission(role=role_repository_write, permission=permission_repository_write)

    # Implied roles
    roles.new_role_implies(from_role=role_repository_write, to_role=role_repository_read)
    roles.new_role_implies(from_role=role_organization_owner, to_role=role_repository_admin)

    # @TODO: things scoped to resources

    roles.enable(oso)
    policy = """
    allow(actor, action, resource) if
      Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    # Some users
    leina = User(name="Leina")
    steve = User(name="Steve")

    osohq = Organization(id="osohq")
    oso_repo = Repository(id="oso", public=False, org=osohq)
    some_issue = Issue(id="fix_all_the_bugs", public=False, repo=oso_repo)

    roles.assign_role(leina, oso_repo, role_repository_read)
    # direct assignment to a role on the resource with the permission
    #assert(oso.is_allowed(leina, "read", oso_repo))
    # direct assignment to a role on the parent with the permission
    #assert(oso.is_allowed(leina, "read", some_issue))

    roles.assign_role(steve, oso_repo, role_repository_write)
    # Implied role on same resource.
    assert(oso.is_allowed(steve, "read", oso_repo))

    ###################### NOTES #######################

    ## ROLE DEFINITION

    # does the user need to be a Python class?
    # probably want to support dicts and strings/ints for users and resources
    # if we do that need to figure out how to make the "id" fields default to the object itself
    # oso.create_roles(
    #     user=Actor,
    #     user_id="name",
    #     roles=["ADMIN", "USER"],
    #     # exclusive=True,
    #     # inherits=[("Admin", "User")],
    # )
    # oso.create_roles(user=Actor, resource=Widget, resource_id="id", roles=["OWNER"])
    # role constraints?



    # REPOSITORY PERMISSION
    # permission: (action, resource)
    # where top-level resource is always a tenant

    # Repo permission definitions
#     oso.create_permission_set(resource_type=Repository, actions=["read", "write", "list_issues"])
#
#     # Issue permissions
#     oso.create_permission_set(resource_type=Issue, actions=["read", "write"])
#
#     # Issue relationship
#     oso.add_parent_relationship(name="issue_repo", child=Issue, parent=Repository, get=lambda child: child.repo)
#
#     # Repo roles
#     ## role definition
#     oso.create_role(resource_type=Repository, name="READ")
#     oso.create_role(resource_type=Repository, name="WRITE")
#     oso.create_role(resource_type=Repository, name="ADMIN")
#
#     ## permission assignment
#     oso.add_role_permission(resource_type=Repository, name="READ", permission={"action": "read", "resource": Repository})
#     oso.add_role_permission(resource_type=Repository, name="READ", permission={"action": "list_issues", "resource": Repository})
#     oso.add_role_permission(resource_type=Repository, name="READ", permission={"action": "read", "resource": Issue}, relationship="issue_repo")
#
#     oso.add_role_permission(resource_type=Repository, name="WRITE", permission={"action": "write", "resource": Repository})
#
#     ## role inheritance
#     oso.role_implies(role={"role": "WRITE", "resource_type": Repository}, implies={"role": "READ", "resource_type": Repository})
#
#     # Organization permission definitions
#     oso.create_permission_set(resource_type=Organization, actions=["read", "create_repo", "list_roles", "list_repos"])
#
#     ## role definition
#     oso.create_role(resource_type=Organization, name="OWNER")
#     oso.create_role(resource_type=Organization, name="MEMBER")
#
#     ## permission assignment
#     oso.add_role_permission(resource_type=Organization, name="MEMBER", permission={"action": "read", "resource": Organization})
#     oso.add_role_permission(resource_type=Organization, name="MEMBER", permission={"action": "list_repos", "resource": Organization})
#     oso.add_role_permission(resource_type=Organization, name="MEMBER", permission={"action": "create_repo", "resource": Organization})
#
#     oso.add_role_permission(resource_type=Organization, name="OWNER", permission={"action": "list_roles", "resource": Organization})
#
#     ## implied roles within a single resource type
#     oso.role_implies(role={"role": "OWNER", "resource_type": Organization}, implies={"role": "MEMBER", "resource_type": Organization})
#
#
#     # Resource relationships
#
#     # This still only works in Python (not super clear how to abstract across API)
#     oso.add_parent_relationship(name="repo_org", child=Repository, parent=Organization, get=lambda child: child.org)
#
#     # implied roles across resource types
#     ## if you are an org owner, you are an admin in every repo of the org
#     ## need a relationship if the resource types don't match
#     ## this specific example is defining a base role for the org
#     oso.role_implies(role={"role": "OWNER", "resource_type": Organization}, implies={"role": "ADMIN", "resource_type": Repository}, relationship="repo_org")
#
#
#     # DYNAMIC PERMISSIONS/IMPLIED ROLES
#     ## why? for customizing base roles in GitHub (e.g. custom org member repository permissions)
#     ## the difference between these calls and the ones above is the `scope` argument, which adds the permission to the role only within the given scope
#
#     # customize the ADMIN repository roles within Organization 1 based on the Org settings
#     oso.add_role_permission(scope=org_1, role={"name": "ADMIN", "resource_type": "Repository"}, permission={"action": "delete", "resource": Issue})
#
#     # customize the MEMBER organization role within Organization 1 based on the Org settings
#     oso.add_implied_role(scope=org_1, role={"name": "MEMBER", "resource_type": "Organization"}, implies={"name": "WRITE", "resource_type": Repository})
#     oso.add_role_permission(scope=org_1, role={"name": "MEMBER", "resource_type": "Organization"}, permission={"action": "create_private_repo", "resource": Repository})
#
#
#
#
#     # Evaluating permissions
#
#     ## options for evaluation:
#     # OPTION 1: 2 separate evaluation steps:
#     #   1) evaluate role permissions in the library
#     #   2) check the policy for additional rules
#     # OPTION 2: Go straight to the VM and hook into the role permission evaluation from Polar
#
#
#     # in Notion, I can have admin on a page, but be denied access to a page inside that page if it's private
#
#     polar="""
#     allow(user, action, resource) if
#         Roles.role_allows(user, action, resource);
#
#     deny(user, action, resource: Repository) if
#         resource.secret and
#         not user = resource.created_by;
#
#     allow(user, action, resource: Repository) if
#         resource.restricted and
#         cut and
#         Roles.direct_role_allows(user, action, resource);
#
#
#     # allow someone to see a repo only if they have a repo role assigned, NOT if they have an organization role that would normally give them access to the repo.
#
#     """
#
#     polar="""
#     allow(user, action, resource) if
#         Roles.role_allows(user, action, resource);
#
#     # for public repos, anyone has the "READ" role permissions (implied role based on conditions)
#     allow(user, action, resource: Repository) if
#         resource.public and
#         action in Roles.get_actions({resource_type: Repository, role: "READ"});
#
#
#     """
#
#
#     # NOTES
#     ########
#     # problems with above
#     # - hard to understand the relationships between everything because it's flat
#     # - very redundant
#     # - easy to make a typo/mistake because everything is a string
#     # - actions are now used in 3 places: calls to `is_allowed`, the Polar policy, and the role permission assignments.
#     #   So, if the name of an action is changed, it potentially has to change in 3 places (vs. 2 before)
#     #       - potential way to improve this is to make it possible to imply roles based on conditions, rather than on a role
#
#     # open questions
#     # - how do we handle public/private? For simple stuff like that it would be really useful to add conditions on role-permission assignments
#     #       - could also use deny logic in the policy or check role permissions in the policy so you can add a condition
#     # - role constraints? mutually exclusive? How do we deal with this?
#     #       - roles that are hierarchical are probably also mutually exclusive
#     #       - use cases that only have global roles might not want mutually exclusive roles
#     # - do we need to deal with implied roles not being static? This is like Notion's "restricted page" feature. Basically would require us to keep track of
#     #   a separate organization base/implied role for each repo. This is just a dynamic base role basically.
#
#
#
#     # Role Model
#     # Permissions are tied to a resource type.
#     # Roles are tied to a resource type.
#     # Roles can have permissions that match their resource type.
#     # - eg Repository roles can have Repository permissions
#     # Roles can have permissions that match a child resource type, as long
#     # as that child resource type doesn't have roles of it's own.
#     # - eg Repository roles can have Issue permissions as long as
#     #      there are no Issue roles.
#     #
#     # Two mutually exclusive roles can not be implied by the same role.
#
#
#
#
#
#
#     # TODO:
#     # -[x] define API for creating roles
#     # -[x] define API for creating, adding/removing permissions to roles
#     # -[x] define API for defing resource relationships
#     # -[] think through UX for devs and what features this supports in the frontend
#     # -[] how do dynamic permissions get evaluated with `is_allowed?`
#
#     # Monday TODO:
#     # - [ ] Put rest of the data into the data model.
#     # - [ ] Start evaluating things, see how it goes. (make sure model supports all the stuff we need)
#     # - management api calls
#     # - is_allowed
#     # - [ ] Improve configuration UX
#     # - opt in to more complexity but be easy to use for simple stuff (non dynamic) too.
#     # - figure out evaluation in sqlalchemy + list filtering
#
#     ### BRAINSTORMING
#
#     # {user: the_actor, role: "ADMIN", resource: the_widget, kind: "Widget"}
#
#     role_config = """
#     GlobalRole = {
#         user: Actor,
#         user_id: "name"
#         roles: ["ADMIN", "USER"]
#     }
#
#     WidgetRole = {
#         user: Actor,
#         user_id: "name",
#         resource: Widget,
#         resource_id: "id"
#         roles: ["OWNER"]
#     }
#
#
#     scope resource: Widget {
#         allow(user, action, resource) if
#             has_role(user, "ADMIN", resource) and
#
#
#         allow_role("ADMIN", _action, resource: Widget);
#         allow_role("MEMBER", action, resource: Widget) if
#             action in ["read", "write"];
#
#         allow(user: Actor, action, resource: Widget) if
#             allow_role()
#
#     }
#     """
#
#     rules = """
#     # permissions on roles
#     allow(user: Actor, action, widget: Widget) if
#         role = Roles.get_user_roles(user) and
#         role.has_perm(action, widget) and
#         not widget.private;
#
#     # could also just evaluate role permissions in the library, with no hook from Polar, and introduce deny logic to Polar
#
#
#
#     allow(user: Actor, "UPDATE", widget: Widget) if
#         {role: "ADMIN"} in Roles.get_user_roles(user) or
#         {role: "OWNER"} in Roles.get_user_roles(user, widget) or
#         widget.public;
#
#     role_allow(role: {role: "OWNER", resource: resource}, _action, resource: Widget);
#
#     role_allow(role: WidgetRole, _action, resource: Widget) if
#         role.widget = resource;
#
#     role_allow(role: {role: "OWNER", resource: resource.parent}, _action, resource: Widget);
#
#
#     #allow(user: Actor, "UPDATE", widget: Widget) if
#     #    Roles.user_in_role(user, {role: "ADMIN"}) or
#     #    Roles.user_in_role(user, "OWNER", widget) or
#     #   widget.public;
#
#     allow(user: Actor, "UPDATE", resource: Widget) if
#         {role: "OWNER"} in Roles.user_roles(user, resource.parent);
#
#     #allow(user: Actor, "UPDATE", resource: Widget) if
#     #    Roles.user_in_role(user, role, resource.parent) and
#     #    role_allow(role, action, resource);
#
#     allow(user: Actor, action, resource: Widget) if
#         allow(user, action, resource.parent);
#
#     allow(user: Actor, _action, resource: WidgetParent) if
#         Roles.user_in_role(user, "ADMIN", resource);
#     """
#
#     # need to know
#     # User / Actor class
#     # Resources the user can have roles on.
#     #
#
#     roles = Roles()
#
#
# ## Static vs dynamic
#
# ### STATIC (Polar?)
# #### What Role types exist
# #### Role inheritance
# #### What permissions exist
# #### Custom logic
#
# ### DYNAMIC (DB)
# #### Role instances
# #### User-role assignments
# #### Role-permission assignments
# #### Role-resource relationships
#
# ## What changes a lot?
# # custom role creation
# # user-role-resource
#
# ## What doesn't change a lot?
# # role-permission
# # Role levels
# # role scopes/types
# # permissions that exist