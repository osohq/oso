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

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

#### New features

##### Custom query timeouts

Added the ability for users to configure query timeouts using a `POLAR_TIMEOUT_MS` environment variable. To disable timeouts (which is useful for debugging), set `POLAR_TIMEOUT_MS` to `0`.

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
