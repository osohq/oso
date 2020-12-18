==============
Builtin Roles
==============

oso includes a library that makes it easy to add roles to your application.
It integrates with our orm library integrations and allows you to create role
relationships between your user models and your resource models.
We then generate some polar to make it easy to define rules over these roles instead
of directly over your users and provide helper methods for managing roles.

Currently this feature is in preview for the SQLAlchemy library.

.. toctree::
    :maxdepth: 1

    sqlalchemy

How it works
============

Lets say you have a User class and a Widget class and you want to assign users a role for a widget like
"OWNER" or "User". Using the roles library you can generate a WidgetRole model which allows you to assign
a user a role for a Widget. The library creates the table and adds methods to allow you to manage assigning
users to the role. You can then write rules over the role instead of the user.

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

Frameworks
==========

To learn more about this feature and see usage examples, see our ORM specific documentation:
  - :doc:`SQLAlchemy </getting-started/builtin-roles/sqlalchemy>`

More framework integrations are coming soon - join us on Slack_ to discuss your
use case or open an issue on GitHub_.

.. _Slack: http://join-slack.osohq.com/
.. _GitHub: https://github.com/osohq/oso
