---
title: "Build Role-Based Access Control (30 min)"
weight: 1
---

# Build Role-Based Access Control (30 min) {{< earlyAccess textSize="sm">}}

When managing access to resources within an application, it can be
useful to group permissions into **roles**, and assign these roles to
users. This is known as **Role-Based Access Control (RBAC).** The Oso
Roles library provides a configuration-based approach to adding
role-based access control to your application. The library is in early
access.

The roles library includes:

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
  by writing custom policies using Polar, Oso's declarative policy
  language.

## Get started

{{% ifLang "python" %}}

{{< tryInFramework >}}

If you are looking for the the currently released version of the sqlalchemy-oso library, you can
find it [here](/reference/frameworks/sqlalchemy).

{{% /ifLang %}}

{{% ifLang not="python" %}}
The Oso Roles library is coming soon for {{< lang >}}! {{< /lang >}}

For now, you can [check out the library documentation for Python]({{< ref path="/new-roles" lang="python" >}}) or read our [guide on role modeling in Polar](/learn/roles).

If you want to get roles working in your app now or just want to
register your interest for an Oso Roles library in {{< lang >}} [drop into our Slack](http://join-slack.osohq.com) or
<a href="mailto:engineering@osohq.com?subject=Roles%20support%20for%20{{< currentLanguage >}}&body=I%27m%20interested%20in%20Oso%20roles%20support%20for%20{{< currentLanguage >}}">send an email</a>
to our engineering team and we'll unblock you.
{{% /ifLang %}}
