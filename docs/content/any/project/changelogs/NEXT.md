---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso 0.26.2`

### Java

#### Other bugs & improvements.

A new `loadFilesFromResources` API has been added to allow loading policy source code from resource files contained in your packaged `.jar`. Special thanks to [`@kovacstamasx`](https://github.com/kovacstamasx) for this contribution.

### Python

#### Other bugs & improvements

- Resolved an `IndexError` exception in `sqlalchemy-oso` Data Filtering. (thanks to @jackdreillyvia for the contribution)
- Resolved a false-negative in `sqlalchemy-oso` Data Filtering when comparing ORM objects. (thanks to @jackdreillyvia for the contribution)

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
