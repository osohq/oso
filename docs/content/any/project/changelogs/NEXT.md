---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso` NEW_VERSION

### Core

#### Other bugs & improvements

- Fixed a bug where a negated constraint on a dot lookup could cause Polar to crash
  when the underlying variable became bound.
- Removed syntax for parenthesized specializers like `f(_: (x));`, which don't
  currently achieve anything.

### Python

#### Other bugs & improvements
- Thanks to [Clara McCreery](https://github.com/chmccreery) for a correction to our
  Python data filtering docs!

## `RELEASED_PACKAGE_1` NEW_VERSION

### Node.js

#### Other bugs & improvements
- The `Class` type for representing abstract resources for data filtering is
  now a top-level export.

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
