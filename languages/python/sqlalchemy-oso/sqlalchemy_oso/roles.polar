# TODO(gj): hard-coded SQLAlchemy role lookups.
actor_role(actor, role) if
    role in actor.repo_roles or
    role in actor.org_roles;

testing(role, [implied_role, resource], roles) if
    print("LOOKING FOR ROLE ON", resource, "THAT IMPLIES", implied_role) and
    # print("testing", implied_role, resource, roles) and
    [name, config] in roles and
    implied_role in config.implies and
    role = [name, resource] or
    ancestor_testing(role, [name, resource]);

ancestor_testing(role, [implied_role, implied_resource]) if
    ancestor(implied_resource, ancestor) and
    print("LOOKING FOR ROLE ON", ancestor, "THAT IMPLIES", implied_role, "ON", implied_resource) and
    resource(ancestor, _, _, roles) and
    # print("ancestor_testing", implied_role, implied_resource) and
    resource(implied_resource, namespace, _, _) and
    # print("     looking for", ":".join([namespace, implied_role]), ancestor) and
    testing(role, [":".join([namespace, implied_role]), ancestor], roles);

allow(actor, action, resource) if
    resource(resource, namespace, actions, roles) and

    # 'action' is valid for 'resource'
    action in actions and

    # Role grants local permission (action & role defined in same namespace).
    role_has_permission([role, role_resource], action, resource, roles) or

    # Role grants non-local permission (action & role defined in different namespaces).
    ancestor_role_has_permission([role, role_resource], ":".join([namespace, action]), resource) and

    (
        # print("direct", role, role_resource) and
        actor_role(actor, assigned_role) and
        role = assigned_role.name and
        role_resource = assigned_role.resource
    ) or (
        resource(role_resource, _, _, role_resource_roles) and
        testing([implied_role, implied_role_resource], [role, role_resource], role_resource_roles) or
        ancestor_testing([implied_role, implied_role_resource], [role, role_resource]) and
        # print("implied", implied_role, implied_role_resource) and
        actor_role(actor, assigned_role) and
        implied_role = assigned_role.name and
        implied_role_resource = assigned_role.resource
    );

role_has_permission([name, resource], action, resource, roles) if
    [name, config] in roles and
    action in config.permissions;

ancestor_role_has_permission(role, action, resource) if
    ancestor(resource, ancestor) and
    resource(ancestor, _, _, roles) and
    role_has_permission(role, action, ancestor, roles);

role_implies_permission([name, resource], implied, resource, roles) if
    [name, config] in roles and
    implied in config.implies;

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
