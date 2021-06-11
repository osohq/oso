# TODO(gj): hard-coded SQLAlchemy role lookups.
actor_role(actor, role) if
    role in actor.repo_roles or
    role in actor.org_roles;

allow(actor, action, resource) if
    resource(resource, _, actions, _) and
    action in actions and # 'action' is valid for 'resource'
    # print(action, resource) and
    role_with_direct_permission(required_role, [action], resource) and
    # required_role = [required_role_name, required_role_resource] and
    # print("  required ->", required_role_name, required_role_resource) and

    actor_role(actor, assigned_role) and
    # print("    assigned ->", assigned_role.name, assigned_role.resource) and

    implied_role(implied_role, required_role, resource) and
    implied_role = [implied_role_name, implied_role_resource] and
    # print("      implied =>", implied_role_name, implied_role_resource) and

    implied_role_name = assigned_role.name and
    # print("      matches  ==>", implied_role_name, assigned_role.name) and
    # print("      checking ==>", implied_role_resource, assigned_role.resource) and
    implied_role_resource = assigned_role.resource;

# checking direct permission
role_with_direct_permission(role, [action], resource) if
    resource(resource, namespace, _, roles) and
    (
        parent(resource, parent_resource) and
        role_with_direct_permission(role, [namespace, action], parent_resource)
    ) or (
        # print("roles =>", roles) and
        [role_name, role_details] in roles and
        action in role_details.permissions and
        role = [role_name, resource]
    );

# checking parent
# TODO(gj): I think I can drop this definition
role_with_direct_permission(role, [namespace, action], resource) if
    (
        parent(resource, parent_resource) and
        role_with_direct_permission(role, [namespace, action], parent_resource)
    ) or (
        resource(resource, _, _, roles) and
        [role_name, role_details] in roles and
        ":".join([namespace, action]) in role_details.permissions and
        role = [role_name, resource]
    );

# A role implies itself.
implied_role(role, role, _);

implied_role(implied_role, [role, role_resource], resource) if
    # print("  checking parent for", namespace, role) and
    parent(resource, parent_resource) and
    # print("  parent", resource, parent_resource) and
    implied_role(implied_role, [role, role_resource], parent_resource);

# checking local implications
implied_role(implied_role, [role, resource], resource) if
    resource(resource, _namespace, _, roles) and
    # print("roles =>", roles) and
    # print("        checking local implications for", role, resource) and
    [role_name, role_details] in roles and
    # print("          checking local role", role_name) and
    role in role_details.implies and
    # print("    ", namespace, role_name, "implies", namespace, role) and
    implied_role(implied_role, [role_name, resource], resource);

# checking non-local implications
implied_role(implied_role, [role, role_resource], resource) if
    # not resource = role_resource and
    # TODO(gj): should this be role_resource?
    resource(resource, _namespace, _, roles) and
    resource(role_resource, role_namespace, _, _) and
    # print("        checking non-local implications for", role_namespace, role) and
    [role_name, role_details] in roles and
    # print("          checking if", role_name, resource, "implies", role, role_resource) and
    ":".join([role_namespace, role]) in role_details.implies and
    # print("    ", namespace, role_name, "implies", role_namespace, role) and
    implied_role(implied_role, [role_name, resource], resource);
