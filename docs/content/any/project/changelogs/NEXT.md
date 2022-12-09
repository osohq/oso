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

- Thank you to [`FinnRG`](https://github.com/FinnRG) and
  [`onalante-msft`](https://github.com/onalante-msft) for updating dependencies
  across the core, the C API crate, and the Rust language library.
- Use `$crate` references in `polar_core` macros to allow macro usage
  without a `use polar_core::*;` glob import.
- Fix `Value::String` serialization bug that allows string injection.
- Fix `Operator::Dot` serialization bug that breaks member access with
  keys that cannot be used unquoted (e.g. `obj.("a b")`).
