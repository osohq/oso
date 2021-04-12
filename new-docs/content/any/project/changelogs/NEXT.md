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

#### Breaking Changes

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

- Behavior of `a(x) if x;` has changed:
  - It's now equivalent to `a(x) if x == true;`.
  - It now works if `x` is unbound.

### Rust

#### Other bugs & improvements

- The Rust CLI now uses [`clap`](https://crates.io/crates/clap) to expose a
  prettier interface thanks to
  [@joshrotenberg](https://github.com/joshrotenberg) via [PR
  #828](https://github.com/osohq/oso/pull/828).
 - Added `FromPolar` and `ToPolar` implementations for more `std::collections` types.
  Thanks to [`@gjvnq`](https://github.com/gjvnq) for [PR #822](https://github.com/osohq/oso/pull/822)!

### Node.js

#### Other bugs & improvements

- Added `free()` method to enable manually freeing the underlying Polar WASM
  instance. This should *not* be something you need to do during the course of
  regular usage. It's generally only useful for scenarios where large numbers
  of instances are spun up and not cleanly reaped by the GC, such as during a
  long-running test process in 'watch' mode.

- The Polar `Variable` type is now exposed in the Node.js library, allowing users to pass unbound variables to `queryRule()` and `isAllowed()`.

```js
const oso = new Oso();
await oso.loadStr('hello("world"); hello("something else");');
const query = oso.queryRule("hello", new Variable("var"));
for await (const result of query) {
  console.log(result);
}

=> Map(1) { 'var' => 'world' }
=> Map(1) { 'var' => 'something else' }
```

### Go

#### Other bugs & improvements

- Go lib no longer tries to print the zero values it uses for bookkeeping. This would crash when running on macOS under delve.

### OTHER_LANGUAGE

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
