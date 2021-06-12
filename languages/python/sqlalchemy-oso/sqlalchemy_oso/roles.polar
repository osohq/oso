# TODO(gj): hard-coded SQLAlchemy role lookups.
actor_role(actor, role) if
    role in actor.repo_roles or
    role in actor.org_roles;

allow(actor, action, resource) if
    resource(resource, namespace, actions, _) and
    action in actions and # 'action' is valid for 'resource'
    role_grants_permission(required_role, [namespace, action], resource) and
    actor_role(actor, assigned_role) and
    implied_role([implied_role_name, implied_role_resource], required_role, resource) and
    implied_role_name = assigned_role.name and
    implied_role_resource = assigned_role.resource;

role_grants_permission(role, namespaced_action, resource) if
    parent(resource, parent) and
    role_grants_permission(role, namespaced_action, parent);

# Role grants local permission (action & role defined in same namespace).
role_grants_permission(role, [namespace, action], resource) if
    resource(resource, namespace, _, roles) and
    [name, config] in roles and
    action in config.permissions and
    role = [name, resource];

# Role grants non-local permission (action & role defined in different namespaces).
role_grants_permission(role, [namespace, action], resource) if
    resource(resource, resource_namespace, _, roles) and
    not namespace = resource_namespace and
    [name, config] in roles and
    ":".join([namespace, action]) in config.permissions and
    role = [name, resource];

# A role implies itself.
implied_role(role, role, _);

implied_role(implied_role, [role, role_resource], resource) if
    parent(resource, parent_resource) and
    implied_role(implied_role, [role, role_resource], parent_resource);

# checking local implications
implied_role(implied_role, [role, resource], resource) if
    resource(resource, _, _, roles) and
    [name, config] in roles and
    role in config.implies and
    implied_role(implied_role, [name, resource], resource);

# checking non-local implications
implied_role(implied_role, [role, role_resource], resource) if
    # not resource = role_resource and
    resource(resource, _, _, roles) and
    resource(role_resource, role_namespace, _, _) and
    [name, config] in roles and
    ":".join([role_namespace, role]) in config.implies and
    implied_role(implied_role, [name, resource], resource);
