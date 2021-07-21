
## Relationships
# - `has_role` (predicate)
# - `has_permission` (predicate)
# - `has_role` (application method on a user or group)
# - parent (attribute on resources)
# - user in group (`teams` attr on user)

# Hook into allow
allow(actor, action, resource) if has_permission(actor, action, resource);


############################
# Relationship definitions #
############################

## Relationships that need to be cleared when registering:
    # - repo.org
    # - issue.repo
    # - user.has_role
    # - user.teams

# User-role mapping to application data
has_role(user: User, role: String, resource: OsoResource) if
    res = user.has_role(role, resource) and
    res = true;

# Group-role mapping to application data
has_role(group: Team, role: String, resource: OsoResource) if
    group.has_role(role, resource);


# Role implication from group role to user role
has_role(u: User, role: String, resource: OsoResource) if
    # TBD: move this into has_group?
    team in u.teams and
    team matches Team and   # DO NOT REMOVE: this check is necessary to avoid infinite recursion
    has_role(team, role, resource);

# define role by attribute relationship
has_role(u: User, "owner", org: Org) if
    u = org.owner;

# Ownership
# option 1 (just use role relationships)
# has_role(u: User, "owner", issue: Issue) if
#     issue.created_by = u;

# # option 2 (custom relationship)
owns(u: User, issue: Issue) if
    issue.created_by = u;

#############################
# Relationship implications #
#############################

# Org policy #
##############

# Role-based permissions
has_permission(u: User, "invite", o: Org) if has_role(u, "owner", o);
has_permission(u: User, "delete_repo", o: Org) if has_role(u, "owner", o);
has_permission(u: User, "create_repo", o: Org) if has_role(u, "member", o);
has_permission(u: User, "list_repos", o: Org) if has_role(u, "member", o);


# Org role implications
has_role(u: User, "member", o: Org) if has_role(u, "owner", o);

# Repo policy #
###############

# Role-based permissions
has_permission(u: User, "pull", r: Repo) if has_role(u, "reader", r);
has_permission(u: User, "list_issues", r: Repo) if has_role(u, "reader", r);
has_permission(u: User, "push", r: Repo) if has_role(u, "writer", r);
has_permission(u: User, "create_issue", r: Repo) if has_role(u, "writer", r);

# Attribute-based permissions
has_permission(_: User, "view", repo: Repo) if repo.is_public;

# Repo role implications (related)
has_role(u: User, "reader", r: Repo) if has_role(u, "member", r.org);
has_role(u: User, "admin", r: Repo) if has_role(u, "owner", r.org);

# Repo role implications (local)
has_role(u: User, "reader", r: Repo) if has_role(u, "writer", r);
has_role(u: User, "writer", r: Repo) if has_role(u, "admin", r);

# Issue policy #
################

# Issue permissions
has_permission(u: User, "read", i: Issue) if has_role(u, "reader", i.repo);
has_permission(u: User, "edit", i: Issue) if has_role(u, "writer", i.repo);
has_permission(u: User, "delete", i: Issue) if has_role(u, "admin", i.repo);
has_permission(u: User, "delete", i: Issue) if owns(u, i);



# TODO:
# - [ ] Make `OsoResource`, `OsoActor`, `OsoGroup` base classes
# - [ ] Divide KB by namespaces
#       - "prototypes" namespace to query for constraints
#       - "data" namespace for mapping to app data? (maybe this is just under relationships)
#       - "relationships" namespace for setting up relationships
# - [ ] Figure out if/how role variants will be specified/defined
# - [ ] Finish validation checks
#       - How to check rule bodies?
#       - asserts -> errors
#       - Validate that `has_role` is only being called with valid roles
# - [ ] Write some validation tests
# - [ ] Write some policy tests

# UX issues:
# - [ ] role implications don't reference the user, but writing them this way requires including the user


# Notes from debugging this policy:
# - [ ] it's hard to trace from a query that should pass (e.g. permission) to all the things that should allow it
#        - honestly a killer dev tool for policy inspection would really help
#        with this: if I could see all the relationshps that lead to
#        has_permission(user, "push", repo) that would be fantastic