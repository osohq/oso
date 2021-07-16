---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `sqlalchemy-oso` NEW_VERSION

### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

#### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

### New features

#### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

### Other bugs & improvements

- Thanks to [`@tomashozman`](https://github.com/tomashozman) for cleaning up
  some SQLAlchemy imports ([#997](https://github.com/osohq/oso/pull/997)).

## `oso` NEW_VERSION

### Core

#### Breaking changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Singleton variables changed from warnings to errors

[Singleton variables](https://docs.osohq.com/reference/polar/polar-syntax.html#singletons) occur only once in a rule.
Polar now considers them an error unless they're explicitly marked with an underscore.

*Before:*

```polar
f(x, y, z) if y = z; # issues a warning for x
```

*After:*

```polar
# f(x, y, z) if y = z; # would cause a parse error

f(_x, y, z) if y = z; # write this instead!
```

#### New features

##### Custom query timeouts

Added the ability for users to configure query timeouts using a `POLAR_TIMEOUT_MS` environment variable. To disable timeouts (which is useful for debugging), set `POLAR_TIMEOUT_MS` to `0`.

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().

### Ruby (`oso-oso`)

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

#### New features

##### Roles in Ruby

The ruby library now has
[built-in support for Role-Based Access Control (RBAC) policies](/guides/roles),
which you can turn on with `OSO.enable_roles`.

#### Other bugs & improvements

- Oso's ruby library now behaves better with code reloading in development. You
  can use `OSO.register_class(Klass)` and calls to `foo matches Klass` will
  always use the up-to-date version of the `Klass` constant, even if it's been
  reloaded.
