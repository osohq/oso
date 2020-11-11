=======================================================
Role-based Access Control for Application Authorization
=======================================================

When managing access to resources within an application, it can be useful to
group permissions into **roles**, and assign these roles to users. This is
known as **Role-based Access Control (RBAC).**

While there is no one way to implement RBAC, there are two types of
relationships that must be considered in a roles system:

1. **User-role relationships** define what roles a user has. The relationship
could be a direct assignment from user-to-role, but could also be an indirect
relationship that depends on additional information.
2. **Role-permission relationships** define the access permissions that a
role grants to a user. In oso, permissions generally consist of an **action**
and a **resource.**

oso represents these relationships as rules in a policy file, written in our
declarative logic programming language called **Polar**. In general, a policy
specifies user-role relationships with ``user_in_role`` rules and
role-permission relationships with ``role_allow`` rules. The general form of
an **allow rule** that calls these rules looks like this:

.. code-block:: polar
    :class: no-select

    allow(actor, action, resource) if
        user_in_role(actor, resource, role) and
        role_allow(role, action, resource);

The rest of this document explains how to implement these rules for different RBAC use cases.

.. Benefits of RBAC (TODO)

Global Roles
============

"Global" roles refers to a single set of roles that applies to the entire
application. Roles are not scoped to a particular domain, such as a tenant or
a resource. Global roles are often useful in single-tenant applications that
require a small set of roles

User-role mappings
-------------------

Static mappings between users and roles can be specified in Polar. This
avoids implementing user-role mappings in your application code, but does
mean that role assignments must be hardcoded for all users.

This example assumes that users are stored as a ``User`` model, but any
object can be used to represent a user (including a simple string).

.. code-block:: polar
    :class: no-select

    # user_in_role defines roles for users
    user_in_role(user: User{username: "steve"}, "admin");
    user_in_role(user: User{username: "leina"}, "admin");

    # assigning groups of users to the same role
    user_in_role(user: User, "admin") if
        user.username in ["steve", "leina", "alex", "sam"];

To avoid hardcoding role assignments, which may be useful if you expect to
assign new users to roles dynamically, you can store user-role assignments in
your application, and look up the assignment in the policy:

.. code-block:: polar
    :class: no-select

    # Get role assignment from user object
    user_in_role(user: User, role) if
        role = user.role and
        valid_role(role);

    # Declare valid roles
    valid_role(role: String) if role in ["admin", "member"];

Role-permission mappings
-------------------------

Role permissions are defined in Polar with ``role_allow`` rules. These are
very similar to ``allow`` rules, but instead of taking an actor as the first
argument, they take a role.

.. code-block:: polar
    :class: no-select

    # allow the admin to take any action on any resource
    role_allow("admin", _action, _resource);

    # allow the user role to read and write to any resource
    role_allow("member", action: String, _resource: BlogPost) if
        action in ["read", "write"];

Enabling roles
--------------

In order to use roles in your application, define an ``allow`` rule that uses
the role logic you've defined, and query it using the ``is_allowed()`` method
in the oso library.

.. code-block:: polar
    :class: no-select

    # allow rule to enable role checking
    allow(actor: User, action, resource) if
        user_in_role(actor, role) and
        role_allow(role, action, resource);

With the ``allow`` role defined, you can query it using the oso library:

.. code-block:: python
    :class: no-select

    @app.route('/blog_post/<int:id>', methods=["GET"])
    def get_blog_post(request) if
        post = get_blog_post(id)
        oso.is_allowed(request.user, "read", post)

Roles in a multi-tenant application
===================================

In multi-tenant applications, roles are usually scoped to only apply to users
and resources within a particular tenant.


One-to-many tenant-user and tenant-resource relationships
---------------------------------------------------------

A straight-forward multi-tenant RBAC system has the following characteristics:

- Users and resources can only belong to a single tenant
- The same set of roles exists for all tenants
- Roles have the same permissions for all tenants (e.g. ``admin`` in tenant_1
    provides the same access control rights as it does in tenant_2, but users in
    tenant_1 cannot access resources in tenant_2).

A role model that meets the above characteristics is very similar to the model for [Global Roles]().

User-role mappings and role-permission mappings can be done the same way as
[Global Roles](), with ``user_in_role`` and ``role_allow``.

All that is required to scope roles to single tenants is to check tenancy in
the ``allow`` rule that implements the role check.

.. code-block:: polar
    :class: no-select

    # User-role mappings
    user_in_role(User{username: "steve"}, "admin");
    user_in_role(User{username: "leina"}, "admin");

    # Role-permission mappings
    role_allow(role: "admin", _action, resource);

    # `allow` rule to enable role checking, with tenant scoping
    allow(actor: User, action, resource) if
        actor.tenant = resource.tenant and
        user_in_role(actor, role) and
        role_allow(role, action, resource);

The above check will ensure that the user's role will only apply to resources
within the same tenant as the user. This model requires that the tenant is
accessible on both user and resource objects.

Many-to-many tenant-user relationships
--------------------------------------

In some applications, users can belong to multiple tenants, and may have
different roles in each tenant. An example of this is GitHub, where users can
belong to multiple organizations, and may have a different role in each
organization.

User-role mappings
^^^^^^^^^^^^^^^^^^

In this case, mapping users to roles actually becomes mapping users to roles
and tenants. This can be done entirely in the policy with
``user_in_role_for_tenant`` rules. This approach avoids needing to store any
role data in the application, but does mean that role assignments are
hardcoded for all users.

.. code-block:: polar
    :class: no-select

    # Per-tenant user-role mappings
    user_in_role_for_tenant(user: User{name: "leina"}, "admin", tenant_id: 1);
    user_in_role_for_tenant(user: User{name: "leina"}, "member", tenant_id: 2);
    user_in_role_for_tenant(user: User{name: "steve"}, "admin", tenant_id: 2);

To avoid hardcoding role assignments for users, the user-role-tenant
assignments can be stored as application data. One implementation of this
would be to store the roles on the user. Since users can have different roles
depending on the tenant, roles should be stored by tenant.

.. code-block:: polar
    :class: no-select

    # Per-tenant user-role mappings, looked up from application data
    user_in_role_for_tenant(user: User, role, tenant_id: Integer) if
        role = user.get_role_by_tenant(tenant_id);

Role-permission mappings
^^^^^^^^^^^^^^^^^^^^^^^^^

As long as roles have the same permissions across all tenants, ``role_allow``
rules can be used to specify role-permission mappings, as with single-tenant roles.

.. code-block:: polar
    :class: no-select

    # allow the admin to take any action on any resource
    role_allow("admin", _action, _resource);

If the roles have different permissions depending on the tenant, the
``role_allow`` rule can be modified to take the tenant as an argument:

.. code-block:: polar
    :class: no-select

    # allow the admin role for tenant 1 to take any action on Tenant1Resource resources
    role_allow_for_tenant("admin", _action, _resource: Foo, tenant_id: 1);

    # allow the admin role for tenant 2 to take any action on Tenant2Resource resources
    role_allow_for_tenant("admin", _action, _resource: Bar, tenant_id: 2);

Enabling roles
^^^^^^^^^^^^^^

To enable the above rules, write an allow rule that calls ``user_in_role`` to
get the relevant role, and call ``role_allow``. The tenant ID of the resource
is used to look up the role, to make sure that the role is associated with
the same tenant as the resource the actor is trying to access.

.. code-block:: polar
    :class: no-select

    # `allow` rule to enable role checking, with tenant scoping
    allow(actor: User, action, resource) if
        user_in_role_for_tenant(actor, role, resource.tenant_id) and
        role_allow(role, action, resource);

Role Hierarchies
================

Role hierarchies represent a model where certain roles are senior to others.
More senior roles inherit permissions from less senior roles. For example, an
organization may have a "manager" role and a "programmer" role. The "manager"
role is more senior than the "programmer", and therefore it inherits the
permissions of the "programmer" role, in addition to its own permissions.

With roles represented as strings in oso policies, role inheritance can be
represented with the following structure:

.. code-block:: polar
    :class: no-select

    # Grant a role permissions that it inherits from a more junior role
    role_allow(role, action, resource) if
        inherits_role(role, junior_role) and
        role_allow(junior_role, action, resource);

    # Managers inherit all permissions provided by the "engineer" role.
    inherits_role(_senior_role: "manager", _junior_role: "programmer");

By adding the above ``role_allow``, any role hierarchies declared with
``inherits_role`` rules will be enforced. Permissions should be assigned to
roles directly using ``role_allow`` rules:

.. code-block:: polar
    :class: no-select

    # Members can read any resource
    role_allow("programmer", _action, resource: ProgrammingResource);

    # Admins can create and delete resources
    role_allow("manager", _action, resource: ManagerResource);

With these roles in place, users with the "manager" role will be able to take
any action on both programming resources and manager resources.

Adding a new role to the hierarchy is very simple with this structure. For
example, adding an "admin" role that inherits permissions from the "manager"
role would require adding one rule:

.. code-block:: polar
    :class: no-select

    inherits_role("admin", "manager");

Multiple Inheritance
--------------------

This role hierarchy structure supports **multiple inheritance,** meaning that
a single role can inherit from multiple junior roles (by adding more
``inherits_role`` rules). For example, there may be a "test_engineer" role
that the "manager" also inherits permissions from. Simply adding another
``inherits_role`` for "manager" will implement this model.

.. code-block:: polar
    :class: no-select

    inherits_role("manager", "test_engineer");

Resource-specific roles
=======================

When controlling access to more than one type of resource, it is often useful
to use roles that specifically apply to one resource or another. For example,
in a project management app there might be ``Project`` resources, which have
the following roles: "member", "developer", and "manager". These roles assign
permissions specifically to the ``Project`` resource.

If these roles are pre-defined, they generally will confer the same
permissions across all ``Project`` resources, but the users assigned to the
role will differ from project-to-project. In other words, the role-permission
mappings are specific to the resource `type`, while the user-role mappings
are specific to the resource `instance`.

This model can be implemented in Polar by implementing
``user_in_role_for_resource`` and ``role_allow`` rules, which are enabled
with the following top-level ``allow`` rule.

.. code-block:: polar
    :class: no-select

    allow(user, action, resource) if
        user_in_role_for_resource(user, role, resource) and
        role_allow(role, resource);

User-role mappings
------------------

Users are generally assigned a resource-specific role on a per-resource
basis. Meaning, a user could have the "member" role for Project 1 and the
"admin" role for Project 2, and the user's access would be different for each
resource. Users can be mapped to roles on a per-resource basis in Polar, by
hardcoding the user-role-resource assignments:

.. code-block:: polar
    :class: no-select

    # Assign leina the "member" role for Project 1
    user_in_role_for_resource(user: User{name: "leina"},
                                                        role: "member",
                                                        project: Project{id: 1});

To avoid hardcoding the user-role-resource assignments, the assignments can
be stored as application data and accessed from the policy.

There are a variety of ways to store these mappings in the application. The
following rules show how the mapping might be accessed in different ways,
depending on the mapping implementation.

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

Scoping the permissions of a role to a single resource type is
straight-forward in Polar, using rule specializers.

.. code-block:: polar
    :class: no-select

    role_allow("member", "view", _resource: Project);

Resource Hierarchies/ Nested Resources
--------------------------------------

It is common for resources to be nested inside of other resources. To
propagate access control through a resource hierarchy, it can be useful to
use a role to grant access to the top-level resource, and infer permissions
for nested resources based on that role. For example, there may be
``Document`` resources nested within the ``Project`` resource, and the
``Project`` "member" role should also grant certain kinds of access to
documents within the project.

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

Using roles with user groups
============================

Assigning roles to User groups
------------------------------

Sometimes it is helpful to assign a role to a group of users, rather than an
individual user. A good example of this is GitHub. In GitHub, users within an
Organization can be added to Teams. Roles can be assigned to teams, rather
than users, and the access granted by a team-level role applies to all the
team members. For this example, let's say that team-level roles are scoped to
resources.

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

Applications often represent organization hierarchies by creating
hierarchical user groups. For example, GitHub supports nested Teams.
Recursive ``group_in_role`` rules can be used to propagate roles through a
group hierarchy.

.. code-block:: polar
    :class: no-select

    # Groups inherit roles from their parent groups
    group_in_role_for_resource(group: Team, role, resource: Repository) if
        group_in_role_for_resource(group.parent_group, role, resource);

Implied roles
=============

Sometimes it is convenient for user-role relationships to be implied, rather
than direct. For example, in GitHub's permissions system, the user who owns
an organization or repository is assigned the "admin" role for that resource
by default.

Implied role assignments eliminate the need to keep direct user-role mappings
up to date in the event that the data they depend on changes. E.g., if the
ownership of a repository is switched, the "admin" role should automatically
be reassigned to the new owner.

This can be implemented in Polar by adding conditions to the body of
``user_in_role`` rules.

.. code-block:: polar
    :class: no-select

    user_in_role_for_resource(user: User, "admin", resource: Repository) if
        user = resource.owner;
