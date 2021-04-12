allow(actor, action, resource) if
    role_allow(actor, action, resource);

role_allow(actor, action, resource) if
    assume_role(actor, role) and
    has_permission(role, action, resource);

# get all possible roles
assume_role(actor, role) if
    # python version
    # user_role in OsoRoles.get_actor_roles(actor) and
    # user_role.user = actor and

    # sqlalchemy version
    (
        user_role in actor.repository_roles or
        user_role in actor.organization_roles
    ) and

    role_implies(user_role, role);

# role implies itself
role_implies(role, role);

# child role
role_implies(role, implied) if
    relationship(role.resource, child_resource, role_map) and
    [role.name, implied_role] in role_map and
    implied = {
        name: implied_role,
        resource: child_resource
    };


# role directly has permission
has_permission(role, action, resource) if
    role.resource = resource and
    hack_type_check(role.resource, resource_class) and
    role_has_permission(role.name, action, resource_class);

# check for direct permission
role_has_permission(role_name, action, resource_class) if
    role(resource_class, definitions, _implies) and
    [role_name, role_perms] in definitions and
    action in role_perms;

# check for permission via implied map
role_has_permission(role_name, action, resource_class) if
    role(resource_class, _definitions, implies) and
    [role_name, implied_role] in implies and
    role_has_permission(implied_role, action, resource_class);


#### Internal hacks
hack_type_check(_: Organization, Organization);
hack_type_check(_: Repository, Repository);