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

The `or` operator has had its precedence lowered to be consistent with other
programming languages. Existing policies using `or` should be updated where
necessary to group `or` operations using parentheses:

```polar
foo(a, b, c) if a and b or c;
```

would now be written

```polar
foo(a, b, c) if a and (b or c);
```

We have temporarily made policies which combine `and` and `or` _without_
using parentheses throw errors in order to avoid silent changes.
To silence the error, add parentheses.

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

## `oso` NEW_VERSION

### Core

#### Other bugs & improvements

- The debugger can now break on rule matches.
- Polar reserved words (e.g. `type`, `if`, `debug`) can be used as field and method names in
  dictionaries and objects.

#### TODO: enforcement changelog, if we decide to release it
