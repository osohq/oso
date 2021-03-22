---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso` 0.11.2

### `Oso.query` and others no longer require mutable reference

<<<<<<< HEAD
Many thanks to [Sascha Jullmann](https://github.com/saschajullmann) for
[reporting](https://github.com/osohq/oso/issues/740) and
[fixing](https://github.com/osohq/oso/pull/745) the bug.


## `oso` 0.12.0

### External Comparison in Javascript

Added external comparison operators to Javascript.  Note that you almost
certainly need to (supply a custom equality function to the Polar constructor)[https://docs.osohq.com/v1/js/node/interfaces/types.options.html]
to use this, as the default
behavior for javascript only is equal if they're the same object.
=======
Thank you [Fisher Darling](https://github.com/fisherdarling)
for [pointing out](https://github.com/osohq/oso/issues/773) that many
methods on `oso::Oso` do not require a mutable reference.

With this small change, it is no longer necessary to wrap `oso::Oso` in a
mutex in order to use across threads.

## `RELEASED_PACKAGE_1` NEW_VERSION

### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

#### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

### New features

#### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
>>>>>>> 992b72881dfe528a116630573941eaa816e5900a
