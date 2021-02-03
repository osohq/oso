---
title: RBAC Design Patterns
any: true
aliases: 
    - ../../getting-started/roles/index.html
---
<!-- 
Possibly copy content from [Introduction to Roles](../../getting-started/roles/_index.md)?

How to not be duplicative? This should extend that guide, and elaborate on how we approach
roles in general

-- Copy in the content from the existing roles guide -- -->

# Built-in Role-Based Access Control

Oso includes support for adding roles directly to your application via our ORM
integrations. These features let you declaratively create models to represent
roles in your application, relate these role models to your user and resource
models, and manage them through helper methods. We also generate Polar rules
that you can use in your Oso policies to write rules based on the roles you
defined, instead of writing rules over users directly.

This feature is currently supported in [the SQLAlchemy library]({{< relref
path="reference/frameworks/sqlalchemy" lang="python" >}}). If you want to get started with
SQLAlchemy roles, look at [How roles work in Oso]({{< relref
path="learn/roles/how" lang="python" >}}).

For a more in-depth understanding of roles, check out our guide to [Role-Based
Access Control (RBAC) patterns](learn/roles/patterns).
