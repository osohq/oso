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
- Subsequent unification of incompatibly type-constrained variables will now fail
  correctly.
- The operators `not`, `forall`, `or`, `<`, `<=`, `>`, and `>=` can now be used
  with data filtering.

### Node.js

#### Breaking changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

* Previously we supported `POLAR_LOG=trace` and `POLAR_LOG=1` which enabled verbose "TRACE"-level logging of the execution of queries and their constituent goals within the Polar VM.
* `POLAR_LOG=on` was slightly less verbose than `POLAR_LOG=trace` but still produced a voluminous output which made it hard to parse and follow the execution of a particular query.
* To enable easier query debugging we have broken out `POLAR_LOG` into new discrete `INFO` and `TRACE` levels. Specifying `POLAR_LOG=info` will cause Polar to emit a more concise log output intended to be consumed by developers as they build and debug their Polar policies. The more verbose TRACE output is still available through `POLAR_LOG=trace`. Check out our [documentation](/reference/tooling/tracing.html) for more information on tracing.

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

#### Other bugs & improvements

- Thanks to [`@Kn99HN`](https://github.com/Kn99HN) for adding the
  `acceptExpression` query flag to the Node.js lib!

## `sqlalchemy-oso` `NEW_VERSION`

### Other bugs & improvements

- `scoped_session` now correctly handles a `get_checked_permission` callback that
  returns `None`.

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
