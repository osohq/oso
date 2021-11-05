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

#### Breaking changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Parenthesized variable dereferencing in specializer position is no longer permitted

Previously, Oso would treat parenthesized specializers as variables. This made
it possible to write rules with parameters that referenced each other, like so:

```polar
isa(x, y, x: (y));
```

Now, that parenthesized `(y)` will be a parse error. The same logic can be
written in the body of the rule with the `matches` operator:

```polar
isa(x, y) if x matches y;
```

If you discovered this feature and were using it, please get in touch. We'd
love to hear about your use case!

#### Other bugs & improvements

- Fixed a bug where a negated constraint on a dot lookup could cause Polar to crash
  when the underlying variable became bound.

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

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
