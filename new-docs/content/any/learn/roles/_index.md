---
title: RBAC Design Patterns
any: true
---
<!-- 
Possibly copy content from [Introduction to Roles](../../getting-started/roles/_index.md)?

How to not be duplicative? This should extend that guide, and elaborate on how we approach
roles in general

-- Copy in the content from the existing roles guide -- -->

# Built-in Role-Based Access Control

oso includes support for adding roles directly to your application via our ORM integrations.
These features let you declaratively create models to represent roles in your application,
relate these role models to your user and resource models, and manage them through helper methods.
We also generate Polar rules that you can use in your oso policies to write
rules based on the roles you defined, instead of writing them over users
directly.

This feature is currently supported in the SQLAlchemy library.

If you want to get started with SQLAlchemy roles, look at
Roles with SQLAlchemy.

For a more in-depth understanding of roles, check out our guide to
Role-Based Access Control (RBAC) patterns.
