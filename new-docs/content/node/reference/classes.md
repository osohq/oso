---
title: Application Data
---

## Working with JavaScript Types

oso’s Node.js authorization library allows you to write policy rules over
JavaScript types directly. This document explains how different types of
JavaScript values can be used in oso policies.

**NOTE**: More detailed examples of working with application objects can be found in Policy Examples.

### Objects

You can pass any JavaScript object into oso and access its properties from
your policy (see Application Types).

### Class Instances

Any `new`-able JavaScript object (including ES6-style classes) can be
constructed from inside an oso policy using the New operator if
the constructor (a `class` or `function` that responds to [the new operator](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new))
has been **registered** using the `#registerClass` method. An example of
this can be found here.

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans (see Primitive Types).

### Strings

JavaScript strings are mapped to Polar Strings. JavaScript’s string methods may be called in policies:

```
allow(actor, action, resource) if actor.username.endsWith("example.com");
```

```
class User {
  constructor(username) {
    this.username = username;
  }
}

const user = new User('alice@example.com');
oso.isAllowed(user, 'foo', 'bar').then(
  result => assert(result)
);
```

**WARNING**: Polar does not support methods that mutate strings in place.

### Lists

JavaScript [Arrays](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)
are mapped to Polar Lists. JavaScript’s Array methods may be called in policies:

```
allow(actor, action, resource) if actor.groups.includes("HR");
```

```
class User {
  constructor(groups) {
    this.groups = groups;
  }
}

const user = new User(["HR", "payroll"]);
oso.isAllowed(user, 'foo', 'bar').then(
  result => assert(result)
);
```

**WARNING**: Polar does not support methods that mutate lists in place, unless the list is also returned from the method.

Likewise, lists constructed in Polar may be passed into JavaScript methods:

```
allow(actor, action, resource) if actor.hasGroups(["HR", "payroll"]);
```

```
class User {
  constructor(groups) {
    this.groups = groups;
  }

  hasGroups(other) {
    return other.every(group => this.groups.includes(group));
  }
}

const user = new User(["HR", "payroll"]);
oso.isAllowed(user, 'foo', 'bar').then(
  result => assert(result)
);
```

There is currently no syntax for random access to a list element within a policy;
i.e., there is no Polar equivalent of the JavaScript expression `user.groups[1]`.
To access the elements of a list, you may iterate over it with In (List Membership)
or destructure it with pattern matching.

### Iterables

You may iterate over any [synchronous](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols)
or [asynchronous](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/asyncIterator))
JavaScript iterables using the Polar In (List Membership) operator:

```
allow(actor, action, resource) if "payroll" in actor.getGroups();
```

```
class User {
  getGroups() {
    return ["HR", "payroll"];
  }
}

const user = new User();
oso.isAllowed(user, 'foo', 'bar').then(assert);
```

### Promises

oso will `await` any [Promise](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
and then use the resolved value during evaluation of a policy.

### `null`

The JavaScript `null` value is registered as the Polar constant nil.
If a JavaScript function can return `null`, you may want to compare the
result to `nil`:

```
allow(actor, action, resource) if actor.getOptional() != nil;
```

```
class User {
  getOptional() {
    if someCondition() {
      return someThing;
    } else {
      return null;
    }
 }
```

### Summary

### JavaScript → Polar Types Summary

| JavaScript type

 | Polar type

 |     |
 | --- ||  |  |  |  |  ||  |  |  |  |  |
| number (Integer)

                                               | Integer

                                                                                 |
| number (Float)

                                                 | Float

                                                                                   |
| boolean

                                                        | Boolean

                                                                                 |
| Array

                                                          | List

                                                                                    |
| string

                                                         | String

                                                                                  |
