---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso` `NEW_VERSION`

### Core

#### Other bugs & improvements

- Fixed a variable scope bug affecting the `forall` operator that caused affected
  queries to fail with an `UnhandledPartial` error.

### Node.js

#### Breaking changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Second parameter of Oso.query() API changed from bindings to options

Pre-seeding the Polar VM with bindings for a query is a bit of an advanced use
case, but if you were previously passing bindings to `Oso.query()`:

```js
const bindings = new Map([['x', 1]]);
oso.query('f(x)', bindings);
```

You'll need to update that call to pass `bindings` as a key in the new
`QueryOpts` object:

```js
const bindings = new Map([['x', 1]]);
oso.query('f(x)', { bindings });
```

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
