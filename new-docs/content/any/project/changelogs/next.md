---
title: Release 2021-02-XX
menuTitle: 2021-02-XX
any: true
description: Changelog for Release 2021-02-XX (oso 0.1X.X, sqlalchemy-oso 0.5.1) bug fixes.
draft: true
---

## `sqlalchemy-oso` 0.5.1

### Bug fixes & improvements

Fixed a bug in `get_resource_users_by_role` which meant it would only work
if the roles were defined for a resource called "repository".

Many thanks to [Sascha Jullmann](https://github.com/saschajullmann) for
[reporting](https://github.com/osohq/oso/issues/740) and
[fixing](https://github.com/osohq/oso/pull/745) the bug.

## `oso` 0.1X.X

### Node.js

#### New features

##### Comparing JavaScript application types

Added support for using Polar's comparison operators to compare JavaScript
objects.

Note that Polar equality (`==`) and inequality (`!=`) operations involving JS
objects default to comparing operands with JavaScript's [`==` and `!=`
operators][mdn-loose-equality]. If you wish to use a different equality
mechanism (e.g., [`===`][mdn-strict-equality] or Lodash's
[`isEqual()`][lodash-isEqual]), you can provide a custom `equalityFn` when
initializing Oso:

```js
const { Oso } = require('oso');

let oso = new Oso({ equalityFn: (x, y) => x === y });

// Or...

const isEqual = require('lodash.isequal');

oso = new Oso({ equalityFn: (x, y) => isEqual(x, y) });
```

[lodash-isEqual]: https://lodash.com/docs#isEqual
[mdn-loose-equality]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Equality_comparisons_and_sameness#loose_equality_using
[mdn-strict-equality]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Equality_comparisons_and_sameness#strict_equality_using
