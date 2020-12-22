==================================
Built-in Role-Based Access Control
==================================

oso includes support for adding roles directly to your application via our ORM integrations.
These features let you declaratively create models to represent roles in your application,
relate these role models to your user and resource models, and manage them through helper methods.
We also generate Polar rules that you can use in your oso policies to write
rules based on the roles you defined, instead of writing them over users
directly.

This feature is currently supported in the SQLAlchemy library.


If you want to get started with SQLAlchemy roles, look at
:doc:`/getting-started/roles/sqlalchemy/basic_roles`.

For a more in-depth understanding of roles, check out our :doc:`guide to
Role-Based Access Control (RBAC) </getting-started/roles/rbac>`.

.. toctree::
    :hidden:
    :maxdepth: 1

    Guide to RBAC <rbac>
    sqlalchemy/basic_roles


How it works
============

Lets say you have a ``User`` class and a ``Widget`` class and you want to
assign users a role for a widget like "OWNER" or "User". Using the roles
library you can generate a ``WidgetRole`` model which allows you to assign a
user a role for a ``Widget``. The schema for this new model's table looks like
this.

.. image:: /getting-started/builtin-roles/roles.svg

The ``WidgetRole`` table is a join table between ``User`` and ``Widget`` that
contains additional ``id`` (Integer) and ``name`` (String) attributes. In
SQLAlchemy library we add ``User.widgets`` and ``Widget.users`` as
relationships you can query, as well as ``User.widget_roles`` and
``Widget.roles`` to get the roles directly. We also provide
:py:data:`helper methods<sqlalchemy_oso.roles>` for inspecting and managing roles.

With ``WidgetRole`` defined, you can call :py:meth:`sqlalchemy_oso.roles.enable_roles`
which unlocks a few special Polar rules by loading the following base policy:

.. code-block:: polar
    :caption: :fa:`oso`

    # RBAC BASE POLICY

    ## Top-level RBAC allow rule

    ### The association between the resource roles and the requested resource is outsourced from the rbac_allow
    allow(user, action, resource) if
        resource_role_applies_to(resource, role_resource) and
        user_in_role(user, role, role_resource) and
        role_allow(role, action, resource);

    # RESOURCE-ROLE RELATIONSHIPS

    ## These rules allow roles to apply to resources other than those that they are scoped to.
    ## The most common example of this is nested resources, e.g. Repository roles should apply to the Issues
    ## nested in that repository.

    ### A resource's roles applies to itself
    resource_role_applies_to(role_resource, role_resource);

    # ROLE-ROLE RELATIONSHIPS

    ## Role Hierarchies

    ### Grant a role permissions that it inherits from a more junior role
    role_allow(role, action, resource) if
        inherits_role(role, inherited_role) and
        role_allow(inherited_role, action, resource);

    ### Determine role inheritance based on the `widget_role_order` rule
    inherits_role(role: WidgetRole, inherited_role) if
        widget_role_order(role_order) and
        inherits_role_helper(role.name, inherited_role_name, role_order) and
        inherited_role = new WidgetRole(name: inherited_role_name, widget: role.widget);

    ### Helper to determine relative order or roles in a list
    inherits_role_helper(role, inherited_role, role_order) if
        ([first, *rest] = role_order and
        role = first and
        inherited_role in rest) or
        ([first, *rest] = role_order and
        inherits_role_helper(role, inherited_role, rest));

    # USER-ROLE RELATIONSHIPS

    ### Get a user's roles for a specific resource
    user_in_role(user: User, role, resource: Widget) if
        session = OsoSession.get() and
        role in session.query(WidgetRole).filter_by(user: user) and
        role.widget.id = resource.id;

.. warning::

    The roles base policy loaded by
    :py:meth:`sqlalchemy_oso.roles.enable_roles` calls builtin rules with the
    following name/arity: ``user_in_role/3``, ``inherits_role/2``, and
    ``inherits_role_helper/3``. Defining your own rules with the same name/arity
    may cause unexpected behavior.


With the roles base policy loaded, you can write rules over roles instead of the user:

.. code-block:: polar

  allow_role(_role: WidgetRole{name: "OWNER"}, "UPDATE", _resource: Widget{});

There are some other kinds of rules you can write to influence how roles work in your application.

You can write a rule that maps roles from one resource to apply to another resource. For instance if
you had an Organization model and wanted people who are admins for the organization to be able to UPDATE
all roles you could do that with these two rules.

.. code-block:: polar

  resource_role_applies_to(widget: Widget, org: Organization) if
    widget.organization_id = org.id;

  allow_role(_role: OrganizationRole{name: "ADMIN"}, "UPDATE", _resource: Widget{});

You can write a rule that specifies a hierarchy for roles. For instance if you want admins to
be able to do everything that members can do.

.. code-block:: polar

  organization_role_order(["ADMIN", "MEMBER"])

