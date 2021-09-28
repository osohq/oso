---
title: JavaScript Types in Polar
weight: 2
aliases:
  - /using/libraries/node/index.html
description:
  Reference for using JavaScript types in Polar.
---

[mdn-array]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
[mdn-new]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new
[mdn-iterator]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols
[mdn-asynciterator]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/asyncIterator
[mdn-promise]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise

## Working with JavaScript Types

Oso’s Node.js authorization library allows you to write policy rules over
JavaScript types directly. This document explains how different types of
JavaScript values can be used in Oso policies.

{{% callout "Note" "blue" %}}
More detailed examples of working with application objects can be found in our
[Guides](guides).
{{% /callout %}}

### Objects

You can pass any JavaScript object into Oso and access its properties from your
policy (see [Application Types](guides/policies#instances-and-fields)).

### Class Instances

Any `new`-able JavaScript object (including ES6-style classes) can be
constructed from inside an Oso policy using Polar's [`new`
operator](polar-syntax#new) if the constructor (a `class` or `function` that
responds to JavaScript's [`new` operator][mdn-new]) has been **registered**
using the `oso.registerClass()` method. An example of this can be found
[here](guides/policies#instances-and-fields).

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans
(see [Primitive Types](polar-syntax#primitive-types)).

### Strings

JavaScript strings are mapped to Polar [strings](polar-syntax#strings).
JavaScript’s string methods may be called in policies:

```polar
allow(actor, _action, _resource) if actor.username.endsWith("example.com");
```

```js
class User {
  constructor(username) {
    this.username = username;
  }
}

const user = new User("alice@example.com");
oso.isAllowed(user, "foo", "bar").then(assert);
```

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate strings in place.
{{% /callout %}}

### Lists

JavaScript [Arrays][mdn-array] are mapped to Polar [lists](polar-syntax#lists).
JavaScript’s Array methods may be called in policies:

```polar
allow(actor, _action, _resource) if actor.groups.includes("HR");
```

```js
class User {
  constructor(groups) {
    this.groups = groups;
  }
}

const user = new User(["HR", "payroll"]);
oso.isAllowed(user, "foo", "bar").then(assert);
```

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate lists in place unless the list is
also returned from the method.
{{% /callout %}}

Likewise, lists constructed in Polar may be passed into JavaScript methods:

```polar
allow(actor, _action, _resource) if actor.hasGroups(["HR", "payroll"]);
```

```js
class User {
  constructor(groups) {
    this.groups = groups;
  }

  hasGroups(other) {
    return other.every((group) => this.groups.includes(group));
  }
}

const user = new User(["HR", "payroll"]);
oso.isAllowed(user, "foo", "bar").then(assert);
```

There is currently no syntax for random access to a list element within a
policy; i.e., there is no Polar equivalent of the JavaScript expression
`user.groups[1]`. To access the elements of a list, you may iterate over it
with [the `in` operator](polar-syntax#in-list-membership) or destructure it
with [pattern matching](polar-syntax#patterns-and-matching).

### Iterables

You may iterate over any [synchronous][mdn-iterator] or
[asynchronous][mdn-asynciterator]) JavaScript iterables using Polar's [in
operator](polar-syntax#in-list-membership):

```polar
allow(actor, _action, _resource) if "payroll" in actor.getGroups();
```

```js
class User {
  getGroups() {
    return ["HR", "payroll"];
  }
}

const user = new User();
oso.isAllowed(user, "foo", "bar").then(assert);
```

### Promises

Oso will `await` any [Promise][mdn-promise] and then use the resolved value
during evaluation of a policy.

### `null`

The JavaScript `null` value is registered as the Polar constant
[`nil`](reference/polar/polar-syntax#nil). If a JavaScript function can
return `null`, you may want to compare the result to `nil`:

```polar
allow(actor, _action, _resource) if actor.getOptional() != nil;
```

```js
class User {
  getOptional() {
    return someCondition() ? someThing : null;
  }
}
```

### JavaScript → Polar Types Summary

| JavaScript type    | Polar type |
| ------------------ | ---------- |
| `number` (integer) | `Integer`  |
| `number` (float)   | `Float`    |
| `boolean`          | `Boolean`  |
| `Array`            | `List`     |
| `string`           | `String`   |
