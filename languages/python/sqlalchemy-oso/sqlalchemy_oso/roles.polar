# TODO(gj): hard-coded SQLAlchemy role lookups.
actor_role(actor, role) if
    role in actor.repository_roles or
    role in actor.organization_roles;

normalize_sqlalchemy_role(_: {name: name, resource: resource}, [namespace, name]) if
    resource(resource, namespace, _, _);

allow(actor, action, resource) if
    resource(resource, _, actions, _) and
    action in actions and # 'action' is valid for 'resource'
    print("action", action) and
    role_with_direct_permission(required_role, [action], resource) and

    actor_role(actor, assigned_role) and
    normalize_sqlalchemy_role(assigned_role, normalized_role) and
    normalized_role = [assigned_namespace, assigned_base_role] and
    print("  normalized", assigned_namespace, assigned_base_role) and

    implied_role(implied_role, required_role, resource) and
    implied_role = [implied_namespace, implied_base_role] and
    print("    implied role", implied_namespace, implied_base_role) and

    implied_namespace = assigned_namespace and
    implied_base_role = assigned_base_role and
    print("    resources", assigned_role.resource, resource);

# checking direct permission
role_with_direct_permission(role, [action], resource) if
    resource(resource, namespace, _, roles) and
    [role_name, role_details] in roles and
    action in role_details.permissions and
    role = [namespace, role_name] or (
        parent(resource, parent_resource) and
        role_with_direct_permission(role, [namespace, action], parent_resource)
    );

# checking parent
role_with_direct_permission(role, [action_namespace, action], resource) if
    resource(resource, namespace, _, roles) and
    [role_name, role_details] in roles and
    action in role_details.permissions and
    role = [namespace, role_name] or (
        parent(resource, parent_resource) and
        role_with_direct_permission(role, [action_namespace, action], parent_resource)
    );

# A role implies itself.
implied_role(role, role, _);

implied_role(implied_role, [namespace, role], resource) if
    # print("  checking parent for", namespace, role) and
    parent(resource, parent_resource) and
    # print("  parent", resource, parent_resource) and
    implied_role(implied_role, [namespace, role], parent_resource);

# checking local implications
implied_role(implied_role, [namespace, role], resource) if
    resource(resource, namespace, _, roles) and
    # print("  checking local implications for", namespace, role) and
    [role_name, role_details] in roles and
    # print("    checking local role", role_name, roles) and
    role in role_details.implies and
    implication = [namespace, role_name] and
    # print("    found local implication", namespace, role_name) and
    implied_role = implication or (
        parent(resource, parent_resource) and
        implied_role(implied_role, implication, parent_resource)
    );

# checking non-local implications
implied_role(implied_role, [namespace, role], resource) if
    resource(resource, resource_namespace, _, roles) and
    not namespace = resource_namespace and
    # print("  checking non-local implications for", namespace, role) and
    [role_name, role_details] in roles and
    # print("  checking if", role_name, "implies", namespace, role) and
    ":".join([namespace, role]) in role_details.implies and
    implication = [resource_namespace, role_name] and
    # print("    found non-local implication", resource_namespace, role_name) and
    implied_role = implication or (
        parent(resource, parent_resource) and
        implied_role(implied_role, implication, parent_resource)
    );
