---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## Oso 0.23.1

### Core

#### Other bugs & improvements

- Oso now allows multiple resource blocks to be declared for the same resource type. The declarations from all resource blocks for a given type are merged together before policy evaluation. This permits rules in one block to reference declarations in another and for resource blocks to be composed over multiple files.
- Fixed a data race in our error handling functionality which resulted in truncated error messages.

### Rust

#### Other bugs & improvements

- Implemented `ExternalIsSubclass` query event. Prevents `x matches Foo and x matches Bar`
  from panicking. Instead, this will now correctly fail when `Foo != Bar`.
  Thanks to [`@davepacheco`](https://github.com/davepacheco) for the contribution!


## `RELEASED_PACKAGE_1` NEW_VERSION


### Go

#### Other bugs & improvements
- Added a `SetAcceptExpression` method to the `Query` struct which makes
  it possible to get partially-evaluated terms back from the core.
  This is a step towards data filtering in Go.
  Thanks to [`@joshrotenberg`](https://github.com/joshrotenberg) for the PR!

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

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
