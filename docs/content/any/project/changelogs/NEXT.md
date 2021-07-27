---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `RELEASED_PACKAGE_1` NEW_VERSION

### LANGUAGE (e.g., 'Core' or 'Python' or 'Node.js')

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().

## `oso` NEW_VERSION

### Core

#### Other bugs & improvements

- Native types (`Integer`, `String`, `Dictionary`, etc.) and
  equivalent host objects created with the `new` operator can now
  be unified transparently.
- The debugger can now break on runtime errors.
- The `var` command  in the debugger now automatically maps variable
  names to their temporary bindings.

### Ruby

#### Other bugs & improvements

- Comparison operations on Ruby objects are now fully supported.

### Rust 

#### New features

##### Roles in Rust

The Rust library now has
[built-in support for Role-Based Access Control (RBAC) policies](/guides/roles),
which you can turn on with `.enable_roles()`.

## `flask-oso` NEW_VERSION

### Other bugs & improvements

- Thanks to [`@arusahni`](https://github.com/arusahni) for surfacing and
  documenting a potential gotcha when using `flask-oso` with other Flask
  libraries that rely on `LocalProxy` objects.
