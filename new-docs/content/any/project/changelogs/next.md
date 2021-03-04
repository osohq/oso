---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso` 0.11.2

### `Oso.query` and others no longer require mutable reference

Thank you [Fisher Darling](https://github.com/fisherdarling)
for [pointing out](https://github.com/osohq/oso/issues/773) that many
methods on `oso::Oso` do not require a mutable reference.

With this small change, it is no longer necessary to wrap `oso::Oso` in a
mutex in order to use across threads.

## `RELEASED_PACKAGE_1` NEW_VERSION

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

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
