---
title: Roles for SQLAlchemy Early Access
weight: 1
no_nav: true
---

Oso Roles for SQLAlchemy {{< earlyAccess textSize="sm">}}
===========================================================

We're working on the next major version of the SQLAlchemy roles library.
This library makes it easy to model role based access control in Polar
and enforce it over your SQLAlchemy models.

The library is now available for early access. Our team continues to
iterate on it and is excited to hear your feedback.

The SQLAlchemy roles library includes:

- **Role configuration** - Declarative configuration for roles and
  permissions for each resource. The roles configuration supports
  multi-tenancy, resource-specific roles, and hierarchical roles. Groups
  and custom roles are coming soon.
- **Data management** - Manage user role assignments in your database,
  linking with your resource data.
- **Enforcement** - Enforce authorization consistently throughout your
  application routing and data access layers.
- **End-user configuration** - Expose authorization configuration to
  end users using Oso's role data API.
- **Last-mile customizations** - Extend authorization logic for each resource
  by writing custom policies using **Polar**, Oso's declarative policy
  language.

The SQLAlchemy roles library is accompanied by [GitClub](https://github.com/osohq/gitclub-sqlalchemy-flask-react), our best
practice example app for using Oso with SQLAlchemy.

This section of the documentation contains guides and reference for
using the new SQLAlchemy library. If you'd like to use the currently
released version [see here](/reference/frameworks/sqlalchemy).

{{% callout "Have feedback?" "green" %}}

Have feedback on this documentation or the library itself? It's under
active development. Our engineering team would love to [hear from you in
Slack.](https://join-slack.osohq.com/)

{{% /callout %}}
