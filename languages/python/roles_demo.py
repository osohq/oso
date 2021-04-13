# Demo

## Demo format

# one python file

# multiple levels of functionality
# 1. Built-in org roles (no customization)--basically equivalent to "global roles" in a multi-tenant app
#       - Owner, member, billing
#       - give some permissions
#       - show allow queries
# 2. Same-level implied roles
#       - show having "Owner" implies "Member" and "Billing"
# 3. Relationships + child permissions
#       - Add org-repo relationships
#       - Create repo permissions
#       - Add repo permissions to org
# 4. Add repo roles (different resources can have their own roles)
#       - Admin, Write, Read
#       - set these up as implied roles
#       - give some permissions
#       - show allow queries
#       - **we'll have to show how you take permissions off the org and switch over to implied roles**
# 5. Base repo permissions within an org (Implied roles across resource types based on relationships)
#       - Set up Org member base permissions for repos in org
#       - Org admin base permissions for repos in org
# 6. Customize the Org member role per organization
#       - Toggle for whether members can create private repos
#       - Create new permission
#       - Add scoped permission to member role for an org

## HAVEN'T IMPLEMENTED:

# 7. Customize base repo role for org members per organization
#       - Add scoped implication to the member role for an org
#       - scoped implication is only scoped to the parent (org) not the child (repo)
# 8. Customize base repo role for org members per repo
#       - Add scoped implication to the member role for a repo
#       - scoped implication is scoped to the child (repo)

from oso import Oso, OsoRoles
from dataclasses import dataclass


@dataclass
class User:
    name: str


@dataclass
class Organization:
    id: str


@dataclass
class Repository:
    id: str
    org: Organization


# 1. Built-in org roles (no customization)--basically equivalent to "global roles" in a multi-tenant app
#       - Owner, member, billing
#       - give some permissions
#       - show allow queries
def one():
    ###################### Configuration ######################################
    # Set up oso
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Organization)

    # Set up roles
    roles = OsoRoles(oso)
    roles.register_class(User)
    roles.register_class(Organization)
    roles.register_class(Repository)
    roles.enable()

    # Simple policy that just uses roles
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            org_member: {
                perms: ["create_repo"]
            },
            org_owner: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.load_str(policy)

    # Demo data
    osohq = Organization(id="osohq")

    leina = User(name="Leina")
    steve = User(name="Steve")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, "org_owner")
    roles.assign_role(steve, osohq, "org_member")

    #### Test

    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Oh no, Leina can't create a repo even though she's THE OWNER????
    assert not oso.is_allowed(leina, "create_repo", osohq)

    # We could give the owner role the create_repo permission, but what we really want to say is
    # that owners can do everything members can do. So the owner role implies the member role.


# 2. Same-level implied roles
#       - show having "Owner" implies "Member" and "Billing"
def two():
    ###################### Configuration ######################################
    # Set up oso
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Organization)

    # Set up roles
    roles = OsoRoles(oso)
    roles.register_class(User)
    roles.register_class(Organization)
    roles.register_class(Repository)
    roles.enable()

    # Same policy as before, but now the "org_owner" role implies the "org_member" role
    policy = """
    resource(_resource: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            org_owner: {
                perms: ["invite"],
                implies: ["org_member"]
            },
            org_member: {
                perms: ["create_repo"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.load_str(policy)

    # Demo data
    osohq = Organization(id="osohq")

    leina = User(name="Leina")
    steve = User(name="Steve")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, "org_owner")
    roles.assign_role(steve, osohq, "org_member")

    #### Test

    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Now, Leina can create a repo becuase she's an owner, and inherits the privileges of members
    assert oso.is_allowed(leina, "create_repo", osohq)


# 3. Relationships + child permissions
#       - Add org-repo relationships
#       - Create repo permissions
#       - Add repo permissions to org
def three():
    ###################### Configuration ######################################
    # Set up oso
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Organization)
    oso.register_class(Repository)

    # Set up roles
    roles = OsoRoles(oso)
    roles.register_class(User)
    roles.register_class(Organization)
    roles.register_class(Repository)
    roles.enable()

    # What if we want to control access to repositories inside the organization?
    # Let's define some repository permissions that we'd like to control access to
    policy = """
    resource(_resource: Repository, "repo", actions, _) if
        actions = [
            "push",
            "pull"
        ];
    """
    # We'd like to let org_members pull from and push to all repos in the org
    # In order to assign repo permissions to organization roles, we need to tell Oso
    # how repos and orgs are related
    policy += """
    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    resource(_resource: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            org_owner: {
                perms: ["invite"],
                implies: ["org_member"]
            },
            org_member: {
                perms: ["create_repo", "repo:pull", "repo:push"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.load_str(policy)

    # Demo data
    osohq = Organization(id="osohq")
    oso_repo = Repository(id="oso_repo", org=osohq)

    leina = User(name="Leina")
    steve = User(name="Steve")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, "org_owner")
    roles.assign_role(steve, osohq, "org_member")

    #### Test

    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Leina can create a repo becuase she's an owner, and inherits the privileges of members
    assert oso.is_allowed(leina, "create_repo", osohq)

    # Steve can push and pull from repos in the osohq org because he is a member of the org
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert oso.is_allowed(steve, "push", oso_repo)

    # Leina can push and pull from repos in the osohq org because she is an owner of the org, and therefore has
    # the same privileges as members of the org
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "push", oso_repo)


# 4. Add repo roles (different resources can have their own roles)
#       - Admin, Write, Read
#       - set these up as implied roles
#       - give some permissions
#       - show allow queries
#       - **we'll have to show how you take permissions off the org and switch over to implied roles**
def four():
    ###################### Configuration ######################################
    # Set up oso
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Organization)
    oso.register_class(Repository)

    # Set up roles
    roles = OsoRoles(oso)
    roles.register_class(User)
    roles.register_class(Organization)
    roles.register_class(Repository)
    roles.enable()

    # Add repo roles
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            org_member: {
                perms: ["create_repo"]
            },
            org_owner: {
                perms: ["invite"],
                implies: ["org_member"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            repo_write: {
                perms: ["push"],
                implies: ["repo_read"]
            },
            repo_read: {
                perms: ["pull"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.load_str(policy)

    # Demo data
    osohq = Organization(id="osohq")
    oso_repo = Repository(id="oso_repo", org=osohq)

    leina = User(name="Leina")
    steve = User(name="Steve")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, "org_owner")
    roles.assign_role(steve, osohq, "org_member")

    # Now we can assign Leina and Steve to roles on the repo directly
    roles.assign_role(leina, oso_repo, "repo_write")
    roles.assign_role(steve, oso_repo, "repo_read")

    #### Test

    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Leina can create a repo becuase she's an owner, and inherits the privileges of members
    assert oso.is_allowed(leina, "create_repo", osohq)

    # Steve can push and pull from repos in the osohq org because he is a member of the org
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert not oso.is_allowed(steve, "push", oso_repo)

    # Leina can push and pull from repos in the osohq org because she is an owner of the org, and therefore has
    # the same privileges as members of the org
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "push", oso_repo)


# 5. Base repo permissions within an org (Implied roles across resource types based on relationships)
#       - Set up Org member base permissions for repos in org
#       - Org admin base permissions for repos in org
def five():
    ###################### Configuration ######################################
    # Set up oso
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Organization)
    oso.register_class(Repository)

    # Set up roles
    roles = OsoRoles(oso)

    # These will probably not be needed later but I need them for now.
    roles.register_class(User)
    roles.register_class(Organization)
    roles.register_class(Repository)

    roles.enable()

    # Policy
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            org_member: {
                perms: ["create_repo"],
                implies: ["repo_read"]
            },
            org_owner: {
                perms: ["invite"],
                implies: ["org_member", "repo_write"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            repo_write: {
                perms: ["push"],
                implies: ["repo_read"]
            },
            repo_read: {
                perms: ["pull"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    # Demo data
    osohq = Organization(id="osohq")

    oso_repo = Repository(id="oso", org=osohq)

    leina = User(name="Leina")
    steve = User(name="Steve")
    gabe = User(name="Gabe")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, "org_owner")
    roles.assign_role(steve, osohq, "org_member")

    roles.assign_role(gabe, oso_repo, "repo_write")

    #### Test

    ## Test Org roles

    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Leina can create a repo because she's the OWNER and OWNER implies MEMBER
    assert oso.is_allowed(leina, "create_repo", osohq)

    # Steve can pull from oso_repo because he is a MEMBER of osohq
    # which implies READ on oso_repo
    assert oso.is_allowed(steve, "pull", oso_repo)
    # Leina can pull from oso_repo because she's an OWNER of osohq
    # which implies WRITE on oso_repo
    # which implies READ on oso_repo
    assert oso.is_allowed(leina, "pull", oso_repo)
    # Gabe can pull from oso_repo because he has WRTIE on oso_repo
    # which implies READ on oso_repo
    assert oso.is_allowed(gabe, "pull", oso_repo)

    # Steve can NOT push to oso_repo because he is a MEMBER of osohq
    # which implies READ on oso_repo but not WRITE
    assert not oso.is_allowed(steve, "push", oso_repo)
    # Leina can push to oso_repo because she's an OWNER of osohq
    # which implies WRITE on oso_repo
    assert oso.is_allowed(leina, "push", oso_repo)
    # Gabe can push to oso_repo because he has WRTIE on oso_repo
    assert oso.is_allowed(gabe, "push", oso_repo)


# 6. Customize the Org member role per organization
#       - Toggle for whether members can create private repos
#       - Create new permission
#       - Add scoped permission to member role for an org
def six():
    ###################### Configuration ######################################
    # Set up oso
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Organization)
    oso.register_class(Repository)

    # Set up roles
    roles = OsoRoles(oso)

    # These will probably not be needed later but I need them for now.
    roles.register_class(User)
    roles.register_class(Organization)
    roles.register_class(Repository)

    roles.enable()

    # Add "create_private_repo action"
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo",
            "create_private_repo"
        ] and
        roles = {
            org_member: {
                perms: ["create_repo"],
                implies: ["repo_read"]
            },
            org_owner: {
                perms: ["invite"],
                implies: ["org_member", "repo_write"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            repo_write: {
                perms: ["push"],
                implies: ["repo_read"]
            },
            repo_read: {
                perms: ["pull"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    # Demo data
    osohq = Organization(id="osohq")
    slack = Organization(id="slack")

    oso_repo = Repository(id="oso", org=osohq)

    leina = User(name="Leina")
    steve = User(name="Steve")
    gabe = User(name="Gabe")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, "org_owner")
    roles.assign_role(steve, osohq, "org_member")

    roles.assign_role(gabe, oso_repo, "repo_write")
    roles.assign_role(gabe, slack, "org_member")

    # Add a scoped role permission
    # Slack organization members are also allowed to create private repos
    roles.add_scoped_role_permission(
        scope=slack,
        role_name="org_member",
        perm_name="org:create_private_repo",
    )

    # role_allow(resource: Organization, "member", "create_private_repo") if
    #     role.has_perm("org:create_private_repo");

    #### Test

    ## Test Org roles

    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Leina can create a repo because she's the OWNER and OWNER implies MEMBER
    assert oso.is_allowed(leina, "create_repo", osohq)

    # Steve can pull from oso_repo because he is a MEMBER of osohq
    # which implies READ on oso_repo
    assert oso.is_allowed(steve, "pull", oso_repo)
    # Leina can pull from oso_repo because she's an OWNER of osohq
    # which implies WRITE on oso_repo
    # which implies READ on oso_repo
    assert oso.is_allowed(leina, "pull", oso_repo)
    # Gabe can pull from oso_repo because he has WRTIE on oso_repo
    # which implies READ on oso_repo
    assert oso.is_allowed(gabe, "pull", oso_repo)

    # Steve can NOT push to oso_repo because he is a MEMBER of osohq
    # which implies READ on oso_repo but not WRITE
    assert not oso.is_allowed(steve, "push", oso_repo)
    # Leina can push to oso_repo because she's an OWNER of osohq
    # which implies WRITE on oso_repo
    assert oso.is_allowed(leina, "push", oso_repo)
    # Gabe can push to oso_repo because he has WRTIE on oso_repo
    assert oso.is_allowed(gabe, "push", oso_repo)

    # Gabe can create private repos in Slack because he is a MEMBER
    assert oso.is_allowed(gabe, "create_private_repo", slack)

    # Leina can't create private repos in osohq because it doesn't have that permission
    assert not oso.is_allowed(leina, "create_private_repo", osohq)


if __name__ == "__main__":
    one()
    two()
    three()
    four()
    five()
    six()
    print("it works")


# Notes from Leina:
# - is it confusing to automatically remove the prefix from permissions?
# - naming of predicates isn't ideal (`role_resource` is weird, cause you could have a resource with just permissions)
# - how would someone ever debug this if they're not getting the result they expect? it's completely opaque
#       - debugging python code is a bit more chill
# - what about string/int representations of tenants (thinking of JWTs)
# - feels like there should be some way of letting users know about the existence of dynamic permissions
#       - maybe we really do just need a central view/good introspection?
# - error handling:
#       - no same-named predicates with different number of args
#       - tell user if action doesn't exist for resource
#       - can't assign perms if roles exist for that resource
#       - easy to get permissions and roles confused
