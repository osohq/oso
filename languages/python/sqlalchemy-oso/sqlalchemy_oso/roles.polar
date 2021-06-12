# TODO(gj): hard-coded SQLAlchemy role lookups.
actor_role(actor, role) if
    role in actor.repo_roles or
    role in actor.org_roles;

allow(actor, action, resource) if
    resource(resource, namespace, actions, roles) and

    # 'action' is valid for 'resource'
    action in actions and

    # Role grants local permission (action & role defined in same namespace).
    role_has_permission(role, action, resource, roles) or

    # Role grants non-local permission (action & role defined in different namespaces).
    ancestor_role_has_permission(role, ":".join([namespace, action]), resource) and

    role_implies_permitted_role([implied_role_name, implied_role_resource], role, resource) and
    actor_role(actor, assigned_role) and
    implied_role_name = assigned_role.name and
    implied_role_resource = assigned_role.resource;

role_has_permission([name, resource], action, resource, roles) if
    [name, config] in roles and
    action in config.permissions;

ancestor_role_has_permission(role, action, resource) if
    ancestor(resource, ancestor) and
    resource(ancestor, _, _, roles) and
    role_has_permission(role, action, ancestor, roles);

# A role implies itself.
role_implies_permitted_role(role, role, _);

role_implies_permitted_role(role, implied_role, resource) if
    parent(resource, parent_resource) and
    role_implies_permitted_role(role, implied_role, parent_resource);

# checking local implications
role_implies_permitted_role(role, [implied_role, resource], resource) if
    resource(resource, _, _, roles) and
    [name, config] in roles and
    implied_role in config.implies and
    role_implies_permitted_role(role, [name, resource], resource);

# checking non-local implications
role_implies_permitted_role(role, [implied_role, implied_role_resource], resource) if
    not resource = implied_role_resource and
    resource(resource, _, _, roles) and
    resource(implied_role_resource, implied_role_namespace, _, _) and
    [name, config] in roles and
    ":".join([implied_role_namespace, implied_role]) in config.implies and
    role_implies_permitted_role(role, [name, resource], resource);

ancestor(child, parent) if parent(child, parent);
ancestor(child, grandparent) if parent(child, parent) and ancestor(parent, grandparent);
