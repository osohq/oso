---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso` 0.26.0

### Core

#### Other bugs & improvements
- Fixed a bug affecting runtime type checking on nested object attributes.
  
## `RELEASED_PACKAGE_1` NEW_VERSION

### Python

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### `@polar_class` is deprecated in favor of `Oso#register_class`

The `@polar_class` decorator used to register classes with Polar has been deprecated. To register a class with Polar it is now necessary to use the [`Oso#register_class`](https://docs.osohq.com/reference/api/index.html#oso.Oso.register_class) API.

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
