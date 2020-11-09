.. meta::
   :description: Modeling roles in oso.

==========
Authorization
==========

oso helps developers build authorization into their applications. Roles are a
common way to think about authorization. When thinking about roles in an
authorization context there are two main questions to ask.

* What roles does a user have. A user/role mapping.
* What permissions does that role have for the resource in question. A role/action mapping.

You can think of the general form for an allow rule with roles to work like this.

    .. code-block:: polar
        :class: no-select

        allow(actor, action, resource) if
            user_in_role(actor, resource, role) and
            role_allow(role, action, resource);

We see what role the actor has for this resource, and we see what actions that
role is allowed to take.

User-Role Mapping.
==================

Direct User Mapping.
--------------------

A simple example to get started would be a single admin role. If users are in the admin
role they would have different privileges than other users that aren't in the admin role.
In the simplest case, the admin role can be embedded directly in the policy.

    .. code-block:: polar
        :class: no-select

        # Assigning specific users to the admin role.
        user_in_role(user: User{username: "steve"}, "admin");
        user_in_role(user: User{username: "leina"}, "admin");

        # Assigning groups of users to the same role.
        user_in_role(user: User, "admin") if
            user.username in ["steve", "leina", "alex", "sam"];

        # Assigning users to the admin role based on a flag on the user.
        user_in_role(user: User{is_admin: true}, "admin");


Roles based on app data.
------------------------

Pre defined sets of roles can also be scoped to things.

Tenants are a common scenario. You might have a seperate tenant for every customer
and the admin role for one tenant is distinct to the one for another tenant.

    .. code-block:: polar
        :class: no-select

        user_in_role(actor, role) if
            ... and
            actor.tenant_id = role.tenant_id;

        allow(actor, action, resource) if
            user_in_role(actor, role) and
            role.tenant_id = resource.tenant_id and
            role_allow(role, action, resource);


In some applications, users can belong to multiple tenants, and may have different
roles in each tenant. An example of this is GitHub, where users can belong to multiple
organizations, and may have a different role in each organization.

In this case, mapping users to roles actually becomes mapping users to roles and tenants.
This can be done entirely in the policy with `user_in_role_for_tenant` rules. This approach
avoids needing to store any role data in the application, but does mean that role assignments
are hardcoded for all users.

    .. code-block:: polar
        :class: no-select

        # User-role mappings
        user_in_role_for_tenant(user: User{name: "leina"}, "admin", tenant_id: 1);
        user_in_role_for_tenant(user: User{name: "leina"}, "member", tenant_id: 2);
        user_in_role_for_tenant(user: User{name: "steve"}, "admin", tenant_id: 2);

To avoid hardcoding role assignments for users, roles can be stored on the user. Since
users can have different roles depending on the tenant, roles should be stored by tenant.

    .. code-block:: polar
        :class: no-select

        user_in_role_for_tenant(user: User, role, tenant_id: 1) if
	        role = user.get_role_by_tenant(tenant_id);

TODO: Move the part about permissions for tenant to the next section.

Role Hierarchies
----------------

TODO: The image

Role hierarchies represent a model where certain roles are senior to others. More senior roles inherit permissions from less senior roles. For example, an organization may have a "manager" role and a "programmer" role. The "manager" role is more senior than the "programmer", and therefore it inherits the permissions of the "programmer" role, in addition to its own permissions. 

With roles represented as strings in oso policies, role inheritance can be represented with the following structure:

    .. code-block:: polar
        :class: no-select

        # Grant a role permissions that it inherits from a more junior role
        role_allow(role, action, resource) if
            inherits_role(role, junior_role) and
            role_allow(junior_role, action, resource);

        # Managers inherit all permissions provided by the "engineer" role.
        inherits_role(_senior_role: "manager", _junior_role: "programmer");

 By adding the above `role_allow`, any role hierarchies declared with `inherits_role` rules will be enforced. Permissions should be assigned to roles directly using `role_allow` rules:


    .. code-block:: polar
        :class: no-select

        # Members can read any resource
        role_allow("programmer", _action, resource: ProgrammingResource);

        # Admins can create and delete resources
        role_allow("manager", _action, resource: ManagerResource);


With these roles in place, users with the "manager" role will be able to take any action on both programming resources and manager resources. 

Adding a new role to the hierarchy is very simple with this structure. For example, adding an "admin" role that inherits permissions from the "manager" role would require adding one rule:

    .. code-block:: polar
        :class: no-select

        inherits_role("admin", "manager");


Resource-specific roles
----------------

When controlling access to more than one type of resource, it is often useful to use
roles that specifically apply to one resource or another. For example, in a project
management app there might be `Project` resources, which have the following roles:
"member", "developer", and "manager". These roles assign permissions specifically to
the `Project` resource. 

If these roles are pre-defined, they generally will confer the same permissions across all
`Project` resources, but the users assigned to the role will differ from project-to-project.
In other words, the role-permission mappings are specific to the resource *type,* while the
user-role mappings are specific to the resource *instance.* 

This model can be implemented in Polar by implementing `user_in_role_for_resource` and `role_allow` rules, which are enabled with the following top-level `allow` rule.

    .. code-block:: polar
        :class: no-select

        allow(user, action, resource) if
	        user_in_role_for_resource(user, role, resource) and
	        role_allow(role, resource);


User-role assignments
----------------

Users are generally assigned a resource-specific role on a per-resource basis. Meaning, a user could have the "member" role for Project 1 and the "admin" role for Project 2, and the user's access would be different for each resource. Users can be mapped to roles on a per-resource basis in Polar, by hardcoding the user-role-resource assignments:


    .. code-block:: polar
        :class: no-select

        # Assign leina the "member" role for Project 1
        user_in_role_for_resource(user: User{name: "leina"}, 
            role: "member", 
            project: Project{id: 1});

To avoid hardcoding the user-role-resource assignments, the assignments can be stored as application data and accessed from the policy. 

There are a variety of ways to store these mappings in the application. The following rules show how the mapping might be accessed in different ways, depending on the mapping implementation.

    .. code-block:: polar
        :class: no-select

        # Get the user's role for a specific Project resource
        # Roles are accessed by resource on the user object
        user_in_role_for_resource(user: User, role, project: Project) if
            role = user.get_role_for_resource(project);

        # Alternative to the above
        # Users are accessed by role on the Project object
        user_in_role_for_resource(user: User, role, project: Project) if
            user in project.get_members(role);

        # Alternative to the above
        # Roles are accessed by user on the Project object
        user_in_role_for_resource(user: User, role, project: Project) if
            role = project.get_role(user);

Role-permission mappings
------------------------

Scoping the permissions of a role to a single resource type is straight-forward in Polar, using rule specializers.

    .. code-block:: polar
        :class: no-select

        role_allow("member", "view", _resource: Project);


Resource Hierarchies/ Nested Resources
--------------------------------------

It is common for resources to be nested inside of other resources. To propagate access control through a resource hierarchy, it can be useful to use a role to grant access to the top-level resource, and infer permissions for nested resources based on that role. For example, there may be `Document` resources nested within the `Project` resource, and the `Project` "member" role should also grant certain kinds of access to documents within the project.

    .. code-block:: polar
        :class: no-select

        # Allow a user to "read" a document if they are in the "member" role for the 
        # parent Project
        allow(user, "read", doc: Document) if
            user_in_role(user, "member", doc.project);

        # Alternative to the above
        # User has the same role on a document as they do on the parent Project
        user_in_role_for_resource(user: User, role, doc: Document) if
            user_in_role_for_resource(user, role, doc.Project);

        # Allow members to "read" documents
        role_allow("member", "read", _resource: Document);

Assigning roles to User groups
------------------------------

Sometimes it is helpful to assign a role to a group of users, rather than an individual user. A good example of this is GitHub. In GitHub,  users within an Organization can be added to Teams. Roles can be assigned to teams, rather than users, and the access granted by a team-level role applies to all the team members. For this example, let's say that team-level roles are scoped to resources.

    .. code-block:: polar
        :class: no-select

        # Get the groups for a user
        user_in_group(user, group) if
            group in user.teams;

        # Assign a role to a group
        group_in_role_for_resource(group: Team{name: "backend_team"}, 
                                                                role: "owner", 
                                                                resource: Repository{name: "backend_repo"});

        # Users inherit roles from their groups
        user_in_role_for_resource(user, role, resource) if
            user_in_group(user, group) and
            group_in_role_for_resource(group, role, resource);

Roles within a hierarchy of groups
----------------------------------

Applications often represent organization hierarchies by creating hierarchical user groups. For example, GitHub supports nested Teams. Recursive `group_in_role` rules can be used to propagate roles through a group hierarchy.

    .. code-block:: polar
        :class: no-select

        # Groups inherit roles from their parent groups
        group_in_role_for_resource(group: Team, role, resource: Repository) if
            group_in_role_for_resource(group.parent_group, role, resource);

Role-Permission Mapping.
==============

TODO
(   
    Maybe this doc should only be about user-role mapping and not about permissions/actions,
    Almost all the content in the last draft is about user-role mapping so maybe it's better to just
    have a full doc about that and tackle the permission/action stuff seperately
)

.. admonition:: What's next
    :class: tip whats-next

    * Explore how to :doc:`/getting-started/application/index`.
    * Dig deeper on :doc:`/getting-started/policies/index`.
    * Check out oso in action: :doc:`/using/examples/index`.
    * Explore the :doc:`/more/design-principles` behind oso.

------------------------

.. include:: /newsletter.rst

.. spelling::
   cURL
