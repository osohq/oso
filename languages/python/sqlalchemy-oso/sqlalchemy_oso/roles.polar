# TODO(gj): hard-coded SQLAlchemy role lookups.
actor_role(actor, role) if
    role in actor.repository_roles or
    role in actor.organization_roles;

allow(actor, action, resource) if
    resource(resource, namespace, actions, roles) and
    action in actions and # 'action' is valid for 'resource'
    print("action", action) and
    role_with_direct_permission(required_role, action, resource) and
    role_implies_required_role(implied_role, required_role, resource) and
    print("required", required_role) and
    print("    implied by", implied_role) and
    false and
    actor_role(actor, assigned_role) and
    [role_name, role_details] in roles and (
        action in role_details.permissions and

        required_role(namespace, role_name, resource, required_role) and
        print("required_role", required_role) and
        false
    ) or (
        action in role_details.permissions and
        assigned_role matches {name: role_name, resource: resource} # direct role assignment
    ) or (
        # Check resource-local implications.
        implied_role in role_details.implies and
        action in roles.(implied_role).permissions and
        resource = assigned_role.resource and
        resource(resource, _, _, roles) and
        assigned_role_name = assigned_role.name and
        implied_role in roles.(assigned_role_name).implies
    ) or (
        # Assigned role is for a parent of the resource. Walk ancestry looking
        # for role implication. TODO(gj): currently only one level of checking.
        parent(resource, assigned_role.resource) and
        resource(assigned_role.resource, _, _, assigned_roles) and
        assigned_role_name = assigned_role.name and
        ":".join([namespace, role_name]) in assigned_roles.(assigned_role_name).implies and
        action in role_details.permissions
    );

# checking direct permission
role_with_direct_permission(role, action, resource) if
    not action.__contains__(":") and
    resource(resource, namespace, _, roles) and
    [role_name, role_details] in roles and
    action in role_details.permissions and
    role = ":".join([namespace, role_name]) or (
        parent(resource, parent_resource) and
        role_with_direct_permission(role, ":".join([namespace, action]), parent_resource)
    );

# checking parent
role_with_direct_permission(role, action, resource) if
    action.__contains__(":") and
    resource(resource, namespace, _, roles) and
    [role_name, role_details] in roles and
    action in role_details.permissions and
    role = ":".join([namespace, role_name]) or (
        parent(resource, parent_resource) and
        role_with_direct_permission(role, action, parent_resource)
    );

# checking local implications
role_implies_required_role(implied_role, required_role, resource) if
    [namespace, role] = required_role.split(":") and
    resource(resource, namespace, _, roles) and
    print("  checking local implications for", required_role) and
    (
        parent(resource, parent_resource) and
        role_implies_required_role(implied_role, required_role, parent_resource)
    ) or
    [role_name, role_details] in roles and
    role in role_details.implies and
    implication = ":".join([namespace, role_name]) and
    print("    found", implication) and
    implied_role = implication or (
        parent(resource, parent_resource) and
        role_implies_required_role(implied_role, implication, parent_resource)
    );

# checking non-local implications
role_implies_required_role(implied_role, required_role, resource) if
    [namespace, _] = required_role.split(":") and
    resource(resource, resource_namespace, _, roles) and
    not namespace = resource_namespace and
    print("  checking non-local implications for", required_role) and
    (
        parent(resource, parent_resource) and
        role_implies_required_role(implied_role, required_role, parent_resource)
    ) or
    [role_name, role_details] in roles and
    print("  checking", role_name) and
    required_role in role_details.implies and
    implication = ":".join([resource_namespace, role_name]) and
    implied_role = implication or (
        parent(resource, parent_resource) and
        role_implies_required_role(implied_role, implication, parent_resource)
    );

# implied_role() if
#     implied_role in role_details.implies and (
#         print("implied_role ->", implied_role) and
#         print("\t->", roles.(implied_role)) and
#         action in roles.(implied_role).permissions and
#         role = ":".join([namespace, role_name])
#     ) or (
#         parent(resource, parent_resource) and
#         role_has_permission(role, action, parent_resource)
#     );

    # resource(resource, namespace, _, roles) and
    # [role_name, role_details] in roles and
    # action in role_details.permissions and
    # role = ":".join([namespace, role_name]) or (
    #     parent(resource, parent_resource) and
    #     role_has_permission(role, action, parent_resource)
    # );



# - Does actor have 'role' that permits 'action' on 'resource'?
# - Does actor have 'role2' that implies 'role' that permits 'action' on 'resource'?
# - Does actor have 'role3' on 'resource3' (a parent of 'resource') that implies 'role' or 'role2'?
#   - And so on up the ancestry...
# - Does actor have 'role4' on 'resource3' (a parent of 'resource') that permits 'resource_namespace:action'?
#   - What about >1 hop in the ancestry? E.g., an OrgRole grants 'issue:edit'.


# Find roles that have this permission
# Does the user have a role that implies
