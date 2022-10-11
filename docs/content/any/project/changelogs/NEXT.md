---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `0.26.3` NEW_VERSION

### Node.js (e.g., 'Core' or 'Python' or 'Node.js')

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Fixed type checking of objects and custom checks

Previously, any JavaScript object would type check as a `Dictionary`, and it wasn't possible
to override this with a custom `isa` check.

In this version, only an object explicitly created with `new Dict({x: 1})` will be recognized
as a Polar dictionary.

This fixes the ability to use custom `isa` checks. For example:

```js
  p.registerClass(Object, {
    name: 'Bar',
    isaCheck: (instance) => instance instanceof Object && instance.typename && instance.typename == "Bar"
  })
```

registers a class with Polar named `Bar`, and Polar will consider any object with a field `typename` set to `"Bar"`
as an instance of the type `Bar`.



## `sqlalchemy-oso` 0.26.3

### Other bugs & improvements

- A missing version constraint on the Flask-SQLAlchemy extra allowed
  Flask-SQLAlchemy versions greater than 2.x to be used with `sqlalchemy-oso`.
  The `sqlalchemy-oso` library requires some updates for compatibility with
  Flask-SQLAlchemy 3.0, and progress on those updates will be tracked in
  https://github.com/osohq/oso/issues/1631. Until compatibility with
  Flask-SQLAlchemy 3.0 is achieved, we've added a runtime check on the
  Flask-SQLAlchemy version that will raise an error if an incompatible version
  is found. Thanks to [`@snstanton`](https://github.com/snstanton) for the
  report and PR!
