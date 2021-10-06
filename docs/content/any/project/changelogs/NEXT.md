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

### Node.js

#### Other bugs & improvements

- Fixed a bug preventing dictionaries created in Polar from making the round-trip
  to JS and back.

  Many thanks to [`@rradczewski`](https://github.com/rradczewski) for
  [raising](https://github.com/osohq/oso/issues/1242) and reproducing
  the issue, and confirming the fix!

### Rust

#### Other bugs & improvements

- Changed an internal debugging flag away from using `RUST_LOG` so that
  Rust users wont be flooded with messages that they probably don't want.

### Go

#### Other bugs & improvements

- Fixed a bug that prevented loading multiple files via the `LoadFiles` API.

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
