role_allow(actor, action, resource) if
    resource(resource, namespace, actions, roles) and

    # 'action' is valid for 'resource'
    action in actions and

    # Role grants local permission (action & role defined in same namespace).
    __oso_internal__role_has_permission([role, role_resource], action, resource, roles) or

    # Role grants non-local permission (action & role defined in different namespaces).
    __oso_internal__ancestor_role_has_permission([role, role_resource], __oso_internal_roles_helpers__.join(":", namespace, action), resource) and

    user_in_role(actor, role, role_resource);

user_in_role(actor, role, resource) if
    __oso_internal__role_implies_permitted_role([implied_role_name, implied_role_resource], [role, resource], resource) and
    actor_role(actor, assigned_role) and
    implied_role_name = assigned_role.name and
    implied_role_resource = assigned_role.resource;

__oso_internal__role_has_permission([name, resource], action, resource, roles) if
    [name, config] in roles and
    action in config.permissions;

__oso_internal__ancestor_role_has_permission(role, action, resource) if
    __oso_internal__ancestor(resource, ancestor) and
    resource(ancestor, _, _, roles) and
    __oso_internal__role_has_permission(role, action, ancestor, roles);

# A role implies itself.
__oso_internal__role_implies_permitted_role(role, role, _);

__oso_internal__role_implies_permitted_role(role, implied_role, resource) if
    parent(resource, parent_resource) and
    __oso_internal__role_implies_permitted_role(role, implied_role, parent_resource);

# checking local implications
__oso_internal__role_implies_permitted_role(role, [implied_role, resource], resource) if
    resource(resource, _, _, roles) and
    [name, config] in roles and
    implied_role in config.implies and
    __oso_internal__role_implies_permitted_role(role, [name, resource], resource);

# checking non-local implications
__oso_internal__role_implies_permitted_role(role, [implied_role, implied_role_resource], resource) if
    __oso_internal__ancestor(implied_role_resource, resource) and
    resource(resource, _, _, roles) and
    resource(implied_role_resource, implied_role_namespace, _, _) and
    [name, config] in roles and
    __oso_internal_roles_helpers__.join(":", implied_role_namespace, implied_role) in config.implies and
    __oso_internal__role_implies_permitted_role(role, [name, resource], resource);

__oso_internal__ancestor(child, parent) if parent(child, parent);
__oso_internal__ancestor(child, grandparent) if parent(child, parent) and __oso_internal__ancestor(parent, grandparent);
