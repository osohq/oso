allow(actor, action, resource) if
    resource(resource, namespace, actions, roles) and

    # 'action' is valid for 'resource'
    action in actions and

    # Role grants local permission (action & role defined in same namespace).
    _role_has_permission(role, action, resource, roles) or

    # Role grants non-local permission (action & role defined in different namespaces).
    _ancestor_role_has_permission(role, ":".join([namespace, action]), resource) and

    _role_implies_permitted_role([implied_role_name, implied_role_resource], role, resource) and

    actor_role(actor, assigned_role) and
    implied_role_name = assigned_role.name and
    implied_role_resource = assigned_role.resource;

_role_has_permission([name, resource], action, resource, roles) if
    [name, config] in roles and
    action in config.permissions;

_ancestor_role_has_permission(role, action, resource) if
    _ancestor(resource, ancestor) and
    resource(ancestor, _, _, roles) and
    _role_has_permission(role, action, ancestor, roles);

# A role implies itself.
_role_implies_permitted_role(role, role, _);

_role_implies_permitted_role(role, implied_role, resource) if
    parent(resource, parent_resource) and
    _role_implies_permitted_role(role, implied_role, parent_resource);

# checking local implications
_role_implies_permitted_role(role, [implied_role, resource], resource) if
    resource(resource, _, _, roles) and
    [name, config] in roles and
    implied_role in config.implies and
    _role_implies_permitted_role(role, [name, resource], resource);

# checking non-local implications
_role_implies_permitted_role(role, [implied_role, implied_role_resource], resource) if
    _ancestor(implied_role_resource, resource) and
    resource(resource, _, _, roles) and
    resource(implied_role_resource, implied_role_namespace, _, _) and
    [name, config] in roles and
    ":".join([implied_role_namespace, implied_role]) in config.implies and
    _role_implies_permitted_role(role, [name, resource], resource);

_ancestor(child, parent) if parent(child, parent);
_ancestor(child, grandparent) if parent(child, parent) and _ancestor(parent, grandparent);
