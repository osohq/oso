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
to override this with a custom isa check.

In this version, only an object explicitly created with `new Dict({x: 1})` will be recognised
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
