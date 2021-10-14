---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---


## `RELEASED_PACKAGE_1` NEW_VERSION

### Core

#### Other bugs & improvements

- Fixed the way we build our static library on Linux so it doesn't embed
  musl and instead links to the system c runtime library.
  Languages that depend on the static lib Linux build such as python and go
  should support more platforms now.
- Oso will now issue a warning if there is no `allow` rule in your policy (and
  also no `allow_request` or `allow_field` rules).
- Oso will propose a suggested fix if you forget to write an actor block when
  using resource blocks.
- Oso will now issue a warning if there are resource blocks in your policy but
  no calls to `has_permission` in any rules.
- Fixed a bug which led to `var matches Type` failing when `var` was unbound.

### Node.js

#### Other bugs & improvements

- Fixed a bug preventing dictionaries created in Polar from making the round-trip
  to JS and back.

  Many thanks to [`@rradczewski`](https://github.com/rradczewski) for
  [raising](https://github.com/osohq/oso/issues/1242) and reproducing
  the issue, and confirming the fix!
- Oso now defaults to using Lodash's `isEqual` function when comparing JavaScript values
  for equality.

### Rust

#### Other bugs & improvements

- Changed an internal debugging flag away from using `RUST_LOG` so that
  Rust users wont be flooded with messages that they probably don't want.

### Go

#### Other bugs & improvements

- Fixed a bug that prevented loading multiple files via the `LoadFiles` API.

### Core

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Undefined rule validation

Oso will now raise an error if your policy contains calls to rules which are not defined.

For example this policy which relies on an undefined `is_admin` rule

```
allow(actor, action, resource) if is_admin(actor)
```

will produce the following error:

```
ValidationError: Call to undefined rule "is_admin" at line 1, column 37
```

To resolve these validation errors you can either update the policy to include a
definition for your missing rule, or remove the offending call entirely.
#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
