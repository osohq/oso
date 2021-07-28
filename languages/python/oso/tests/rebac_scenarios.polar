
# SIMPLE

## Writing policies: small example
# - Scenario: a developer wants to add organization roles to their application, and use those roles to grant access to some resources that belong to the organization.
#     - Let's say for only 3 resources, with read/write/delete permissions on each
# - What does Oso provide to help me achieve this, and what do I need to implement?
#     - How do I know which is which?
# - Suppose the steps are:
#     - Map roles to application data
#     - Assign permissions to roles for organization with implications
#     - Map parent relationships to application data
#     - Assign permissions to roles for resources via parent relationship with implications
#     - Add role implications
# - **For each of the above steps:**
#     - What does the developer need to do?
#     - What does Oso do?
#     - How easy is if for the developer to understand what is happening at each step?
#         - what new concepts would the developer need to understand?
#         - what is happening implicitly?
#     - How easy is it for the developer to apply these steps to their own application?
# - How easy would it be to extend this to add groups?


# Map roles to application data

# 1. register resources/actors in Python app (would like to eventually replace this
# with types/some way of marking app types as entity types in Polar)

# ```python
#     o = Oso()
#     o.register_resource(Org)
#     o.register_resource(Repo)
#     o.register_resource(Issue)
#     o.register_actor(User)
# ```

# 2. define mapping from roles to resources
# Oso provides something like this to show users what relationship rules can be defined in the relationships scope definition:
def scope relationships {
    type has_role(actor: OsoActor, role: String, resource: OsoResource);
    type has_permissions(actor: OsoActor, actions: List, resource: OsoResource);
    type in_group(actor: OsoActor, group: OsoGroup);
    type owns(actor: OsoActor, resource: OsoResource);
    # etc...
}

# User defines the mappings inside the relationships scope/namespace

scope relationships {
    # If there is one way to access roles for all resources
    has_role(user: User, role_name: String, resource: OsoResource) if
        user.has_role(user, role_name, resource);

    # If there are multiple ways to access roles for all resources
    has_role(user: User, role_name: String, org: Org) if
        user.has_org_role(user, role_name, org);
    # ... etc.

}

# Notes:
# - No way in this implementation to enumerate roles for each resource, which
# maybe would be the intuitive first step
# - Enums could help with this:
enum OrgRole {
    OWNER,
    MEMBER,
    BILLING
}
role_def(role_name: OrgRole, resource: Org);
has_role(u: User, role_name: Enum, resource: OsoResource) if
    role_def(role_name, resource) and  # would be nice if there were a way to add this as a precondition
    user.has_role(u, role_name, resource);

# - Could also add another type of predicate for defining roles:
role_def(role_name: String, org: Org) if role_name in ["OWNER", "BILLING"];
has_role(u: User, role_name: String, resource: OsoResource) if
    role_def(role_name, resource) and  # would be nice if there were a way to add this as a precondition
    user.has_role(u, role_name, resource);

# The above isn't that simple as it's all raw Polar--it feels like there should
# be a better way for us to let users specify valid roles without requiring them
# to implement the validation checks themselves in Polar

#  Assign permissions to roles for organization with implications

scope relationships {
    # If there is one way to access roles for all resources
    has_role(user: User, role_name: String, resource: OsoResource) if
        user.has_role(user, role_name, resource);

    # If there are multiple ways to access roles for all resources
    has_role(user: User, role_name: String, resource: Resource1) if
        user.has_role_for_resource1(user, role_name);
    # ... etc.

    # Permissions
    has_permissions(u: User, ["read"], o: Org) if has_role(u, "MEMBER", o);
    has_permissions(u: User, ["invite"], o: Org) if has_role(u, "OWNER", o);

    # Local Implications
    has_role(u: User, "OWNER", o: Org) if has_role(u, "MEMBER", o);

}

# Map parent relationships to application data

# - This is done by registering attributes with resources (could create a
# specific "relationships" arg if we wanted to or even be as specific as
# "parent")

# ```python
#     o = Oso()
#     o.register_resource(Org)
#     o.register_resource(Repo, properties={"org": Org})
#     o.register_resource(Issue, properties={"repo": Repo})
#     o.register_actor(User)
# ```

# - The attribiutes are then accessed directly from Polar in implication rules,
# but some validation is done to make sure that only registered properties are
# accessed

    has_role(u: User, "ADMIN", r: Repo) if has_role(u, "OWNER", r.org);

# If we want a cleaner separation between the data and the relationships, we could add a `parent` predicate
def scope relationships {
    # Rule prototype
    is_parent(parent: OsoResource, child: OsoResource);
}
scope relationships {
    is_parent(parent, child: Repo) if o = r.org;

    has_role(u: User, "ADMIN", r: Repo) if
        is_parent(parent, r) and
        has_role(u, "OWNER", parent);
}


#############
# Completed policy for this scenario (this is basically GitClub):

scope relationships {
    # If there is one way to access roles for all resources
    has_role(user: User, role_name: String, resource: OsoResource) if
        user.has_role(user, role_name, resource);

    # ORG POLICY

    # Org Permissions
    has_permissions(u: User, ["read"], o: Org) if has_role(u, "MEMBER", o);
    has_permissions(u: User, ["invite"], o: Org) if has_role(u, "OWNER", o);

    # Org Role Implications
    has_role(u: User, "OWNER", o: Org) if has_role(u, "MEMBER", o);

    # REPO POLICY

    # Repo Permissions
    # ...

    # Repo implications
    has_role(u: User, "ADMIN", r: Repo) if has_role(u, "OWNER", r.org);

    # ISSUE POLICY
    # Issue Permissions
    # ...
    # Issue implications
    # ...
}