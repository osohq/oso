allow(actor, action, resource) if
    role_allow(actor, action, resource);

role_allow(actor, action, resource) if
    assume_role(actor, role, resource) and
    has_permission(role, action, resource);

# get all possible roles
assume_role(actor, implied, resource) if
    # python version
    # user_role in OsoRoles.get_actor_roles(actor) and
    # user_role.user = actor and

    # sqlalchemy version
    (
        user_role in actor.repository_roles or
        user_role in actor.organization_roles
    ) and

    role_implies(user_role, implied, resource);

# A role for a resource implies itself for that resource.
role_implies(role, role, resource) if
    resource = role.resource;

# Implied role on same resource.
role_implies(role, implied, resource) if
    resource = role.resource and
    resource(role.resource, _, _, roles) and
    name = role.name and
    implied_role in roles.(name).implies and
    (
        # Check that implied_role is defined on same resource.
        [implied_role, _] in roles and
        implied = {
            name: implied_role,
            resource: role.resource
        }
    );

# Implied role on child resource.
role_implies(role, implied, resource) if
    parent(resource, parent_resource) and
    resource(parent_resource, _, _, roles) and
    print("role ->", role) and
    print("roles ->", roles) and false;
    # name = role.name and
#     implied_role in roles.(name).implies and
#     [implied_namespace, implied_namespaced_role] = implied_role.split(":") and
    # resource(implied_resource, implied_namespace, _, _) and
    # parent(implied_resource, role.resource) and
    # implied2 = {
    #     name: implied_namespaced_role,
    #     resource: implied_resource
    # } and implied = implied2;
    #     parent(implied_resource, role.resource) and
    #     implied2 = implied
    # ) or (
    #     role_implies(implied2, implied, implied_resource)
    # );

# # direct implication
# role_implies(role, implied, child_resource) if
#     parent_resource = role.resource and
#     print("parent", child_resource, parent_resource) and
#     parent(child_resource, parent_resource) and
#     class_namespace(parent_resource, namespace) and
#     resource(parent_resource, namespace, _, roles) and
#     name = role.name and
#     implied_role in roles.(name).implies and
#     print("implied_role ->", implied_role) and
#     implied = {
#         name: implied_role,
#         resource: child_resource
#     };
#     # ) or (
#     #     # print("recurse <-", maybe_parent_resource) and
#     #     role_implies(role, implied, maybe_parent_resource)
#     # );
#     # ([namespace2, role2] = implied_role.split(":") and
#     # print("@@@@@@@@", role2) and
#     # resource(resource2, namespace2, _, roles2) and
#     # print("<<<<<<<<", roles2) and
#     # class_namespace(resource2, namespace2) and
#     # print(">>>>>>>>", namespace2));
#     # # implied2 = {
#     # #     name: role2,
#     # #     resource: resource2
#     # # }) and
#     # # role_implies(implied2, implied, child_resource);

# role directly has permission
has_permission(role, action, resource) if
    role.resource = resource and
    role_has_permission(role.name, action, resource);

# # role indirectly has permission
# has_permission(role, action, resource) if
#     ancestor(resource, role.resource) and
#     resource(role.resource, _, _, roles) and
#     [role.name, role_details] in roles and
#     implied_role in role_details.implies and
#     print(implied_role) and
#     role_has_permission(implied_role, action, resource);

# check for direct permission
role_has_permission(role_name, action, resource) if
    class_namespace(resource, namespace) and
    resource(resource, namespace, _actions, roles) and
    [role_name, role_details] in roles and
    action in role_details.permissions;

# check for permission via implied map
role_has_permission(role_name, action, resource) if
    class_namespace(resource, namespace) and
    resource(resource, namespace, _actions, roles) and
    [role_name, role_details] in roles and
    implied_role in role_details.implies and
    role_has_permission(implied_role, action, resource);

ancestor(child, ancestor) if parent(child, ancestor);
ancestor(child, ancestor) if
    parent(child, parent) and
    ancestor(parent, ancestor);
