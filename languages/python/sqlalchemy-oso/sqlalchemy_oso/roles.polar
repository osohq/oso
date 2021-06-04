allow(actor, action, resource) if
    role_allow(actor, action, resource);

role_allow(actor, action, resource) if
    assume_role(actor, role, resource) and
    has_permission(role, action, resource);

# get all possible roles
assume_role(actor, role, resource) if
    # python version
    # user_role in OsoRoles.get_actor_roles(actor) and
    # user_role.user = actor and

    # sqlalchemy version
    (
        user_role in actor.repository_roles or
        user_role in actor.organization_roles
    ) and

    role_implies(user_role, role, resource);

# role implies itself
role_implies(role, role, _);

# child role
role_implies(role, implied, child_resource) if
    parent_resource = role.resource and
    parent(parent_resource, child_resource) and
    hack_type_check(parent_resource, resource_class) and
    class_namespace(resource_class, namespace) and
    resource(resource_class, namespace, _, roles) and
    name = role.name and
    implied_role in roles.(name).implies and
    (implied = {
        name: implied_role,
        resource: child_resource
    }) or
    ([namespace2, role2] = implied_role.split(":") and
    implied = {
        name: role2,
        resource: child_resource
    });

# role directly has permission
has_permission(role, action, resource) if
    role.resource = resource and
    hack_type_check(role.resource, resource_class) and
    role_has_permission(role.name, action, resource_class);

# check for direct permission
role_has_permission(role_name, action, resource_class) if
    class_namespace(resource_class, namespace) and
    resource(resource_class, namespace, _actions, roles) and
    [role_name, role_details] in roles and
    action in role_details.permissions;

# check for permission via implied map
role_has_permission(role_name, action, resource_class) if
    class_namespace(resource_class, namespace) and
    resource(resource_class, namespace, _actions, roles) and
    [role_name, role_details] in roles and
    implied_role in role_details.implies and
    role_has_permission(implied_role, action, resource_class);

#### Internal hacks
hack_type_check(_: Organization, Organization);
hack_type_check(_: Repository, Repository);
