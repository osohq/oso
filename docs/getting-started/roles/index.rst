==================================
Built-in Role-Based Access Control
==================================

oso includes a library that makes it easy to add roles to your application.
It integrates with our model library integrations and allows you to create role
relationships between your user models and your resource models.
We then generate some polar to make it easy to define rules over these roles instead
of directly over your users and provide helper methods for managing roles.

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

Lets say you have a User class and a Widget class and you want to assign users a role for a widget like
"OWNER" or "User". Using the roles library you can generate a WidgetRole model which allows you to assign
a user a role for a Widget. The schema for this new model's table looks like this.

.. image:: /getting-started/builtin-roles/roles.svg

It's a join table between
User and Widget that contains some metadata like the name of the role. In SQLAlchemy library we add User.widgets
and Widget.users as relationships you can query, as well as User.widget_roles and Widget.roles to get the roles directly.
We also provide helper methods that make managing assigning users to roles easy.

You can then write rules over the role instead of the user.

.. code-block:: polar

  allow_role(WidgetRole{name: "OWNER"}, "UPDATE", Widget{});

There are some other kinds of rules you can write to influence how roles work in your application.

You can write a rule that maps roles from one resource to apply to another resource. For instance if
you had an Organization model and wanted people who are admins for the organization to be able to UPDATE
all roles you could do that with these two rules.

.. code-block:: polar

  resource_role_applies_to(widget: Widget, org: Organization) if
    widget.organization_id = org.id;

  allow_role(OrganizationRole{name: "ADMIN"}, "UPDATE", Widget{});

You can write a rule that specifies a hierarchy for roles. For instance if you want admins to
be able to do everything that members can do.

.. code-block:: polar

  organization_role_order(["ADMIN", "MEMBER"])

