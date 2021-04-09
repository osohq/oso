# Low level api, describes each pice of data
# Any configuration api must be able to specify all these pieces of data.

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



# Brainstorming configs

################################################################################
# What a json/yml version could look like, doesn't collapse too much.
# Roles = {
#     Organization: {
#         actions = ["invite", "create_repo"],
#         roles = {
#             "MEMBER": {"actions": ["create_repo"], "implied_roles": ["Repository:READ"]},
#             "OWNER": {"actions": ["invite"], "implied_roles": ["MEMBER", "Repository:WRITE"]
#         }
#     },
#     Repository: {
#         actions = ["push", "pull"],
#         roles = {
#             "READ": {"actions": ["pull"]},
#             "WRITE": {"actions": ["push"], "implied_roles": ["READ"]
#         },
#         parent: Organization,
#         parent_selector: lambda repo: repo.org
#     },
# }


################################################################################
# Maybe what role types version could look like
class OrganizationRole(Role):
    resource = Organization
    pass

class OrganizationOwnerRole(OrganizationRole):
    actions = "invite"

class OrganizationMemberRole(OrganizationRole):
    actions = "create_repo"
    implied_by = OrganizationOwnerRole

class RepositoryRole(Role):
    resource = Repository
    parent = Organization
    parent_selector: lambda repo: repo.org

class RepositoryWriteRole(RepositoryRole):
    actions = "push"
    implied_by = OrganizationOwnerRole

class RepositoryReadRole(RepositoryRole):
    actions = "pull"
    implied_by = [OrganizationMemberRole, RepositoryWriteRole]


################################################################################
# Maybe what having metadata classes for the resources (instead of the roles) could look like
class OrganizationResource(OsoResource):
    resource_type = Organization

    actions = ["invite", "create_repo"]

    class MemberRole:
        name = "MEMBER"
        actions = ["create_repo"]

    class OwnerRole:
        name = "OWNER"
        actions = ["invite"]
        implied_roles = [MemberRole, RepositoryResource.WriteRole]

class RepositoryResource(OsoResource):
    resource_type = Repository

    # relationships
    parent = Organization
    parent_selector = lambda repo: repo.org

    # actions
    actions = ["push", "pull"]
    
    # roles = ["READ", "WRITE"]
    # role_mappings = {"READ": "pull", "WRITE", "push"}
    
    # roles
    class ReadRole:
        name = "READ"
        actions = "pull"

    class WriteRole:
        name = "WRITE"
        actions = ["push", IssueResource.actions.DELETE]
        implied_roles = "READ"

class IssueResource(OsoResource):
    resource_type = Issue
    parent = Repository
    parent_selector = lambda issue: issue.repo


    actions = ["edit, delete"]

################################################################################
# Flatter version but less repetitive than low level api
oso.config_roles(
    resource_types = {"org": Organization, "repo": Repository, "issue": Issue},
    relationships = {"repo_org": ("repo", "org", lambda r: r.org), "issue_repo": ("issue", "repo", lambda i: i.repo)},
    permissions = [Permission("org", "invite"), Permission("org", "create_repo"), Permission("repo", "push"), Permission("issue", "delete")],
    roles = [
        Role("org", "OWNER", [Permission("org", "invite")]), 
        Role("org","MEMBER", [Permission("org", "create_repo")], 
        Role("repo","READ", [Permission("repo", "pull")]), 
        Role("repo", "WRITE", [Permission("repo", "push"), Permission("issue", "delete")])
    ]
    implications = [
        Role("org", "OWNER").implies(Role("org", "MEMBER")),
        Role("org", "MEMBER").implies(Role("repo", "WRITE")),
        Role("repo", "WRITE").implies(Role("repo", "READ")),
    ]
)

################################################################################
# Version with the implications on roles
oso.config_roles(
    resource_types = {"org": Organization, "repo": Repository, "issue": Issue}
    relationships = {"repo_org": ("repo", "org", lambda r: r.org), "issue_repo": ("issue", "repo", lambda i: i.repo)}
    permissions = [Permission("org", "invite"), Permission("org", "create_repo"), Permission("repo", "push"), Permission("issue", "delete")]
    roles = [
        Role("org", "OWNER", perms=[Permission("org", "invite")], implies=[Role("org", "MEMBER"), Role("repo", "WRITE")]),
        Role("org","MEMBER", perms=[Permission("org", "create_repo"), implies=[Role("repo", "READ")]], 
        Role("repo","READ", perms=[Permission("repo", "pull")]), 
        Role("repo", "WRITE", perms=[Permission("repo", "push"), Permission("issue", "delete")], implies=[Role("repo", "READ")])]
)

################################################################################
# Version with just namespaced strings?
oso.config_roles(
    resource_types = {"org": Organization, "repo": Repository, "issue": Issue}
    relationships = {"repo_org": ("repo", "org", lambda r: r.org), "issue_repo": ("issue", "repo", lambda i: i.repo)}
    permissions = ["perm:org:invite", "perm:org:create_repo", "perm:repo:push", "perm:issue:delete"]
    roles = [
        Role("role:org:OWNER", perms=["perm:org:invite"], implies=["role:org:MEMBER", "role:repo:WRITE"]),
        Role("role:org:MEMBER", perms=["perm:org:create_repo"], implies=["role:repo:READ"]), 
        Role("role:repo:READ", perms=["perm:repo:pull"]), 
        Role("role:repo:WRITE", perms=["perm:repo:push", "perm:issue:delete"], implies=["role:repo:READ"])]

)

################################################################################
# @NOTE: Winning version so far!
# Version with relationships like that too
oso.config_roles(
    resource_types = {"org": Organization, "repo": Repository, "issue": Issue}
    relationships = {"parent:repo:org": lambda r: r.org, "parent:issue:repo": lambda i: i.repo}
    permissions = ["perm:org:invite", "perm:org:create_repo", "perm:repo:push", "perm:issue:delete"]
    roles = [
        Role("role:org:OWNER", perms=["perm:org:invite"], implies=["role:org:MEMBER", "role:repo:WRITE"]),
        Role("role:org:MEMBER", perms=["perm:org:create_repo"], implies=["role:repo:READ"]), 
        Role("role:repo:READ", perms=["perm:repo:pull"]), 
        Role("role:repo:WRITE", perms=["perm:repo:push", "perm:issue:delete"], implies=["role:repo:READ"])
    ]
)

################################################################################
# Just data now
oso.config_roles(
    resource_types = {"org": Organization, "repo": Repository, "issue": Issue}
    relationships = {"parent:repo:org": lambda r: r.org, "parent:issue:repo": lambda i: i.repo}
    permissions = ["perm:org:invite", "perm:org:create_repo", "perm:repo:push", "perm:issue:delete"]
    roles = [
        {
            "name": "role:org:OWNER",
            "perms": ["perm:org:invite"],
            "implies": ["role:org:MEMBER", "role:repo:WRITE"]
        },
        {
            "name": "role:org:MEMBER", 
            "perms": ["perm:org:create_repo"], 
            "implies":["role:repo:READ"]
        },
        {
            "name": "role:repo:READ",
            "perms": ["perm:repo:pull"],
        },
        {

            "name":"role:repo:WRITE", 
            "perms":["perm:repo:push", "perm:issue:delete"], 
            "implies":["role:repo:READ"]
        }
)