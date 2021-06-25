---
title: "Build Role-Based Access Control"
weight: 1
showContentForAnyLanguage: true
aliases:
  - /python/getting-started/roles/builtin_roles.html
---

# Build Role-Based Access Control

When managing access to resources within an application, it can be
useful to group permissions into **roles**, and assign these roles to
users. This is known as **Role-Based Access Control (RBAC).** The Oso
Roles feature provides a configuration-based approach to adding
role-based access control to your application.

The roles feature includes:

- **Role configuration** - Declarative configuration for roles and
  permissions for each resource. The roles configuration supports
  multi-tenancy, resource-specific roles, and hierarchical roles. Groups
  and custom roles are coming soon.
- **Enforcement** - Enforce authorization consistently throughout your
  application routing and data access layers.
- **Last-mile customizations** - Extend authorization logic for each resource
  by writing custom policies using Polar, Oso's declarative policy
  language.

{{% ifLang "python" %}}
## SQLAlchemy

If you are using SQLAlchemy to manage your application data, you can use the
Oso Roles for SQLAlchemy feature in the `sqlalchemy-oso` framework integration
to additionally handle:

- **Data management** - Manage user role assignments in your database,
  linking with your resource data.
- **End-user configuration** - Expose authorization configuration to
  end users using Oso's role data API.

[Check out the library documentation for SQLAlchemy](./sqlalchemy/getting-started)

## Get started

Continue on to the [getting started guide](./getting-started) to see how to
add Oso Roles to a Python application.

{{% /ifLang %}}

{{% ifLang "node" %}}

## Get started

Continue on to the [getting started guide](./getting-started) to see how to
add Oso Roles to a Node.js application.

{{% /ifLang %}}

{{< ifLangExists >}}
{{% ifLang not="node" %}}
{{% ifLang not="python" %}}

## Get started

The Oso Roles feature is coming soon for {{< lang >}}!

For now, you can check out the feature documentation for [Python]({{< ref path="/guides/roles" lang="python" >}}) or
[Node]({{< ref path="/guides/roles" lang="node" >}}),
or read our [guide on role modeling in Polar](/learn/roles).

If you want to get roles working in your app now or just want to
register your interest for the Oso Roles feature in {{< lang >}} [drop into our Slack](http://join-slack.osohq.com) or
<a href="mailto:engineering@osohq.com?subject=Roles%20support%20for%20{{< currentLanguage >}}&body=I%27m%20interested%20in%20Oso%20roles%20support%20for%20{{< currentLanguage >}}">send an email</a>
to our engineering team and we'll unblock you.
{{% /ifLang %}}
{{% /ifLang %}}
{{% /ifLangExists %}}
