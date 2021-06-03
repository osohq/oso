---
title: Release 2021-05-16
menuTitle: 2021-05-16
any: true
description: >-
  Changelog for Release 2021-05-16 (sqlalchemy-oso-preview) containing new features,
  bug fixes, and more.
draft: true
---

## `sqlalchemy-oso-preview` 0.0.5

### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
This release contains breaking changes. Be sure to follow migration steps
before upgrading.
{{% /callout %}}

We've made some updates to the syntax of the `resource` predicate, used to configure resources + roles for sqlalchemy-oso-preview. The goal of these changes is to improve the readability of the configuration and make the roles
features more intuitive to use.

#### Rename "perms" -> "permissions" in resource roles configuration

The `roles` parameter of the `resource` previously included a field called `perms` to specify the role permissions.
We have renamed this field to `permissions` for clarity.

#### Add namespaces to role names

Previously, we required role names to be globally unique. Now, role names will be internally namespaced, removing the globally unique requirement. Like permissions, the role namespace is the resource name specified in the `resource` predicate. Roles names must be unique within a single resource namespace. Roles associated with other resources must be referenced using the namespace. Roles within the same resource can be referenced without the namespace.

Below is an example `resource` predicate that reflects the above changes. For a more in-depth example, see our [documentation](TODO!!!!) and [example repository](TODO!!!).

```polar
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                # `perms` renamed to `permissions`
                permissions: ["invite"]
            },
            owner: {
                permissions: ["list_repos"],
                # Roles from a different resource are now referenced by namespace
                implies: ["member", "repo:reader"]
            }
        };
```

Link to [migration guide]().

### New features

#### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

### Other bugs & improvements

- Calling `sqlalchemy_oso.SQLAlchemyOso.enable_roles()` more than once will now
  raise an error. There's no need to call the method multiple times.

- Using a global SQLAlchemy declarative base class would previously
  result in some issues when reusing the same base class across multiple
  `sqlalchemy_oso.SQLAlchemyOso` instances, e.g., when running multiple tests
  that construct new `SQLAlchemyOso` instances but reuse the same global
  declarative base class. The issues are now fixed by ignoring internal models
  when verifying that all models descending from the given base class have
  primary keys of the same type. For more on that requirement, see [the version
  0.0.4 changelog](project/changelogs/2021-05-26).
