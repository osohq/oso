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
    roles = OsoRoles()
    roles.enable(oso)

    # Simple policy that just uses roles
    policy = """
    allow(actor, action, resource) if
      Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    # Define permissions
    permission_org_invite = roles.new_permission(resource=Organization, action="invite")
    permission_org_create_repo = roles.new_permission(
        resource=Organization, action="create_repo"
    )

    # Define roles
    role_org_owner = roles.new_role(resource=Organization, name="OWNER")
    role_org_member = roles.new_role(resource=Organization, name="MEMBER")

    # Add permissions to roles
    roles.add_role_permission(role=role_org_owner, permission=permission_org_invite)
    roles.add_role_permission(
        role=role_org_member, permission=permission_org_create_repo
    )
    ###########################################################################

    # Demo data
    osohq = Organization(id="osohq")

    leina = User(name="Leina")
    steve = User(name="Steve")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, role_org_owner)
    roles.assign_role(steve, osohq, role_org_member)

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
    roles = OsoRoles()
    roles.enable(oso)

    # Simple policy that just uses roles
    policy = """
    allow(actor, action, resource) if
      not resource = private and
      Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    ### Basic resource role configuration ###

    ## Organizations

    # Define organization permissions
    permission_org_invite = roles.new_permission(resource=Organization, action="invite")
    permission_org_create_repo = roles.new_permission(
        resource=Organization, action="create_repo"
    )

    # Define organization roles
    role_org_owner = roles.new_role(resource=Organization, name="OWNER")
    role_org_member = roles.new_role(resource=Organization, name="MEMBER")

    # Add permissions to organization roles
    roles.add_role_permission(role=role_org_owner, permission=permission_org_invite)
    roles.add_role_permission(
        role=role_org_member, permission=permission_org_create_repo
    )

    # Implied roles for organizations
    roles.add_role_implies(role_org_owner, role_org_member)

    ## Repositories

    # Define repo permissions
    permission_repo_push = roles.new_permission(resource=Repository, action="push")
    permission_repo_pull = roles.new_permission(resource=Repository, action="pull")

    # Define repo roles
    role_repo_write = roles.new_role(resource=Repository, name="WRITE")
    role_repo_read = roles.new_role(resource=Repository, name="READ")

    # Add permissions to repo roles
    roles.add_role_permission(role=role_repo_write, permission=permission_repo_push)
    roles.add_role_permission(role=role_repo_read, permission=permission_repo_pull)

    # Implied roles for repositories
    roles.add_role_implies(role_repo_write, role_repo_read)

    ### Relationships + cross-resource implications ###

    # organizations are the parent of repos
    roles.new_relationship(
        name="repo_org",
        child=Repository,
        parent=Organization,
        parent_selector=lambda child: child.org,
    )

    # Org "OWNER" role implies repo "WRITE" role for every repo in the org
    roles.add_role_implies(role_org_owner, role_repo_write)
    # Org "MEMBER" role implies repo "READ" role for every repo in the org
    roles.add_role_implies(role_org_member, role_repo_read)

    ###########################################################################

    # Demo data
    osohq = Organization(id="osohq")

    oso_repo = Repository(id="oso", org=osohq)

    leina = User(name="Leina")
    steve = User(name="Steve")
    gabe = User(name="Gabe")

    # Things that happen in the app via the management api.
    roles.assign_role(leina, osohq, role_org_owner)
    roles.assign_role(steve, osohq, role_org_member)

    roles.assign_role(gabe, oso_repo, role_repo_write)

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


if __name__ == "__main__":
    one()
    six()
    print("it works")
