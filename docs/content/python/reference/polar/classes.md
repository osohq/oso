---
title: Python Types in Polar
weight: 2
aliases:
  - /using/libraries/python/index.html
description: |
   Reference for working with Python types in Polar.
---

# Working with Python Types

Oso’s Python authorization library allows you to write policy rules over Python
objects directly. This document explains how different types of Python objects
can be used in Oso policies.

{{% callout "Note" "blue" %}}
More detailed examples of working with application classes can be found in our
[Guides](guides).
{{% /callout %}}

## Class Instances

You can pass an instance of any Python class into Oso and access its methods
and fields from your policy (see [Application
Types](guides/policies#instances-and-fields)).

<!-- TODO(gj): link to API docs. -->

Python instances can be constructed from inside an Oso policy using the
[`new`](polar-syntax#new) operator if the Python class has been **registered**
using either the `register_class()` method or the `polar_class()` decorator. An
example of this can be found [here](guides/policies#instances-and-fields).

## Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans
(see [Primitive Types](polar-syntax#primitive-types)). These map to the Python
`int`, `float`, and `bool` types.

## Strings

Python strings are mapped to Polar [strings](polar-syntax#strings). Python’s
string methods may be accessed from policies:

```polar
allow(actor, _action, _resource) if actor.username.endswith("example.com");
```

```python
user = User()
user.username = "alice@example.com"
assert(oso.is_allowed(user, "foo", "bar))
```

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate strings in place. E.g.,
`capitalize()` will have no effect on a string in Polar.
{{% /callout %}}

## Lists

Python lists are mapped to Polar [lists](polar-syntax#lists). Python’s list
methods may be accessed from policies:

```polar
allow(actor, _action, _resource) if actor.groups.index("HR") == 0;
```

```python
user = User()
user.groups = ["HR", "payroll"]
assert(oso.is_allowed(user, "foo", "bar"))
```

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate lists in place. E.g. `reverse()`
will have no effect on a list in Polar.
{{% /callout %}}

Likewise, lists constructed in Polar may be passed into Python methods:

```polar
allow(actor, _action, _resource) if actor.has_groups(["HR", "payroll"]);
```

```python
class User:
   def has_groups(self, groups):
      """ Check if a user has all of the provided groups. """
         for g in groups:
            if not g in self.groups:
               return False
         return True

user = User()
user.groups = ["HR", "payroll"]
assert(oso.is_allowed(user, "foo", "bar))
```

There is currently no syntax for random access to a list element within a
policy; i.e., there is no Polar equivalent of the Python expression
`user.groups[1]`. To access the elements of a list, you may iterate over it
with [the `in` operator](polar-syntax#in-list-membership) or destructure it
with [pattern matching](polar-syntax#patterns-and-matching).

## Dictionaries

Python dictionaries are mapped to Polar
[dictionaries](polar-syntax#dictionaries):

```polar
allow(actor, _action, _resource) if actor.roles.project1 = "admin";
```

```python
user = User()
user.roles = {"project1": "admin"}
assert(oso.is_allowed(user, "foo", "bar))
```

Likewise, dictionaries constructed in Polar may be passed into Python methods.

## Iterables

You may iterate over any Python
[iterable](https://docs.python.org/3/glossary.html#term-iterable), such as
those yielded by a
[generator](https://docs.python.org/3/glossary.html#term-generator), using
Polar's [`in` operator](polar-syntax#in-list-membership):

```polar
allow(actor, _action, _resource) if "payroll" in actor.get_groups();
```

```python
class User:
   def get_groups(self):
      """Generator method to yield user groups."""
      yield from ["HR", "payroll"]

user = User()
assert(oso.is_allowed(user, "foo", "bar))
```

## `None`

The Python value `None` is registered as the Polar constant nil. If a Python
method can return `None`, you may want to compare the result to `nil`:

```polar
allow(actor, _action, _resource) if actor.get_optional() != nil;
```

```python
class User:
   def get_optional(self):
      """Return something or None."""
      if self.some_condition():
          return self.some_thing
      else:
          return None

user = User()
assert(oso.is_allowed(user, "foo", "bar))
```

## Python → Polar Types Summary

| Python type | Polar type   |
| ----------- | ------------ |
| `int`       | `Integer`    |
| `float`     | `Float`      |
| `bool`      | `Boolean`    |
| `list`      | `List`       |
| `dict`      | `Dictionary` |
| `str`       | `String`     |
