
## Relationships
# - `has_role` (predicate)
# - `has_permission(s)` (predicate)
# - `has_role` (application method on a user or group)
# - parent (attribute on resources)
# - user in group (`teams` attr on user)

# Hook into allow
# allow(actor, action, resource) if
#     has_permission(actor, action, resource);
allow(actor, action, resource) if
    relationships::has_permissions(actor, actions, resource) and
    action in actions;


############################
# Relationship definitions #
############################

# These rules answer the question: where do relationships come from?

## Relationships that need to be cleared when registering:
    # - repo.org
    # - issue.repo
    # - user.has_role
    # - user.teams

def scope relationships {
    type has_role(actor: User, role: String, resource: OsoResource);
    type has_role(actor: User, role: String, resource: Repo);
    type has_role(actor: User, role: String, resource: Org);
    type has_role(actor: Team, role: String, resource: Repo);
    type has_role(actor: Team, role: String, resource: OsoResource);
    type has_permissions(actor: User, actions: List, resource: Org);
    type has_permissions(actor: User, actions: List, resource: Repo);
    type has_permissions(actor: User, actions: List, resource: Issue);
    type in_group(actor: User, group: Team);
    type owns(actor: User, resource: Issue);
}

scope relationships {
    # User-role mapping to application data
    has_role(user: User, role: String, resource: OsoResource) if
        user.has_role(role, resource);

    # User-role mapping to application data for specific role
    has_role(user: User, "owner", org: Org) if
        org.owner = user;

    # Group-role mapping to application data
    has_role(group: Team, role: String, resource: OsoResource) if
        group.has_role(role, resource);

    # User-group mapping to application data
    in_group(user: User, team: Team) if
        team in user.teams and
        team matches Team;   # DO NOT REMOVE: this check is necessary to avoid infinite recursion

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

    # OsoResource policy #
    ######################
    # applies to all resources

    # Role implication from group role to user role
    # has_role(u: User, role: String, resource: OsoResource) if
    #     in_group(u, team) and
    #     has_role(team, role, resource);

    # Org policy #
    ##############

    # Role-based permissions
    has_permissions(u: User, ["invite", "delete_repo"], o: Org) if has_role(u, "owner", o);
    has_permissions(u: User, ["create_repo", "list_repos"], o: Org) if has_role(u, "member", o);


    # Org role implications
    has_role(u: User, "member", o: Org) if has_role(u, "owner", o);

    # Repo policy #
    ###############

    # Role-based permissions
    has_permissions(u: User, ["pull", "list_issues"], r: Repo) if has_role(u, "reader", r);
    has_permissions(u: User, ["push", "create_issue"], r: Repo) if has_role(u, "writer", r);

    has_permissions(u: User, ["invite"], r: Repo) if has_role(u, "writer", r);


    # Attribute-based permissions
    has_permissions(_: User, ["view"], repo: Repo) if repo.is_public;

    # Repo role implications (related)
    has_role(u: User, "reader", r: Repo) if has_role(u, "member", r.org);
    has_role(u: User, "admin", r: Repo) if has_role(u, "owner", r.org);

    # Repo role implications (local)
    has_role(u: User, "reader", r: Repo) if has_role(u, "writer", r);
    has_role(u: User, "writer", r: Repo) if has_role(u, "admin", r);

    # Issue policy #
    ################

    # Issue permissions
    has_permissions(u: User, ["read"], i: Issue) if has_role(u, "reader", i.repo);
    has_permissions(u: User, ["edit"], i: Issue) if has_role(u, "writer", i.repo);
    has_permissions(u: User, ["delete"], i: Issue) if has_role(u, "admin", i.repo);
    has_permissions(u: User, ["delete"], i: Issue) if owns(u, i);
}



# TODO:
# - [x] Make `OsoResource`, `OsoActor`, `OsoGroup` valid specializers
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
# - [x] Write some policy tests

# Open questions
# - How to distinguish between the relationship "definitions" and the "implications"?
#       - Currently the definitions don't have body restrictions (other than
#       registered methods/props), and the implications only call other
#       relationship rules in the body
#       - But the implication rules can still access parents as attributes, but
#       the translation is done in the call
# - Do all relationships need to be represented as predicates (e.g., parent
# relationships are currently dot lookups)

# UX issues:
# - role implications don't reference the user, but writing them this way requires including the user


# Notes from debugging this policy:
# - it's hard to trace from a query that should pass (e.g. permission) to all the things that should allow it
#        - honestly a killer dev tool for policy inspection would really help
#        with this: if I could see all the relationshps that lead to
#        has_permission(user, "push", repo) that would be fantastic