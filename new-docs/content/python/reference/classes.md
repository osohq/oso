---
weight: 1
title: Application Data
---

## Working with Python Objects

oso’s Python authorization library allows you to write policy rules over Python objects directly.
This document explains how different types of Python objects can be used in oso policies.

**NOTE**: More detailed examples of working with application classes can be found in Policy Examples.

### Class Instances

You can pass an instance of any Python class into oso and access its methods and fields from your policy (see Application Types).

Python instances can be constructed from inside an oso policy using the New operator if the Python class has been **registered** using
either the `register_class()` method or the `polar_class()` decorator.
An example of this can be found here.

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans (see Primitive Types).
These map to the Python `int`, `float`, and `bool` types.

### Strings

Python strings are mapped to Polar Strings. Python’s string methods may be accessed from policies:

```
allow(actor, action, resource) if actor.username.endswith("example.com");
```

```
user = User()
user.username = "alice@example.com"
assert(oso.is_allowed(user, "foo", "bar))
```

**WARNING**: Polar does not support methods that mutate strings in place. E.g. `capitalize()` will have no effect on
a string in Polar.

### Lists

Python lists are mapped to Polar Lists. Python’s list methods may be accessed from policies:

```
allow(actor, action, resource) if actor.groups.index("HR") == 0;
```

```
user = User()
user.groups = ["HR", "payroll"]
assert(oso.is_allowed(user, "foo", "bar"))
```

**WARNING**: Polar does not support methods that mutate lists in place. E.g. `reverse()` will have no effect on
a list in Polar.

Likewise, lists constructed in Polar may be passed into Python methods:

```
allow(actor, action, resource) if actor.has_groups(["HR", "payroll"]);
```

```
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

There is currently no syntax for random access to a list element within a policy;
i.e., there is no Polar equivalent of the Python expression `user.groups[1]`.
To access the elements of a list, you may iterate over it with In (List Membership)
or destructure it with pattern matching.

### Dictionaries

Python dictionaries are mapped to Polar Dictionaries:

```
allow(actor, action, resource) if actor.roles.project1 = "admin";
```

```
user = User()
user.roles = {"project1": "admin"}
assert(oso.is_allowed(user, "foo", "bar))
```

Likewise, dictionaries constructed in Polar may be passed into Python methods.

### Iterables

You may iterate over any Python [iterable](https://docs.python.org/3/glossary.html#term-iterable),
such as those yielded by a [generator](https://docs.python.org/3/glossary.html#term-generator),
using the Polar In (List Membership) operator:

```
allow(actor, action, resource) if "payroll" in actor.get_groups();
```

```
class User:
   def get_groups(self):
      """Generator method to yield user groups."""
      yield from ["HR", "payroll"]

user = User()
assert(oso.is_allowed(user, "foo", "bar))
```

### `None`

The Python value `None` is registered as the Polar constant nil.
If a Python method can return `None`, you may want to compare the result
to `nil`:

```
allow(actor, action, resource) if actor.get_optional() != nil;
```

```
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

<!-- ### Summary

### Python → Polar Types Summary

| Python type

 | Polar type

 |     |
 | --- ||  |  |  |  |  ||  |  |  |  |  |  |  |
| int

                                                            | Integer

                                                                                 |
| float

                                                          | Float

                                                                                   |
| bool

                                                           | Boolean

                                                                                 |
| list

                                                           | List

                                                                                    |
| dict

                                                           | Dictionary

                                                                              |
| str

                                                            | String

                                                                                  | -->
