---
title: Ruby Types in Polar
weight: 2
aliases:
  - /using/libraries/ruby/index.html
description: |
  Reference for using Ruby types in Polar.
---

[rb-array]: https://ruby-doc.org/core/Array.html
[rb-array-at]: https://ruby-doc.org/core/Array.html#method-i-at
[rb-enumerable]: https://ruby-doc.org/core/Enumerable.html

## Working with Ruby Types

Oso’s Ruby authorization library allows you to write policy rules over Ruby
objects directly. This document explains how different types of Ruby objects
can be used in Oso policies.

{{% callout "Note" "blue" %}}
More detailed examples of working with application objects can be found in our
[Guides](guides).
{{% /callout %}}

### Class Instances

You can pass any Ruby instance into Oso and access its methods and fields from
your policy (see [Application Types](guides/policies#instances-and-fields)).

Ruby instances can be constructed from inside an Oso policy using the [`new`
operator](polar-syntax#new) if the Ruby class has been **registered** using the
`Oso#register_class` method. An example of this can be found
[here](guides/policies#instances-and-fields).

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans
(see [Primitive Types](polar-syntax#primitive-types)). These map to the Ruby
`Integer`, `Float`, and `TrueClass`/`FalseClass` types.

### Strings

Ruby strings are mapped to Polar [strings](polar-syntax#strings). Ruby’s string
methods may be called in policies:

```polar
allow(actor, _action, _resource) if actor.username.end_with?("example.com");
```

```ruby
class User
  attr_reader :username

  def initialize(username)
    @username = username
  end
end

user = User.new("alice@example.com")
oso.authorize(user, "foo", "bar")
```

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate strings in place.
{{% /callout %}}

### Lists

Ruby [Arrays][rb-array] are mapped to Polar [lists](polar-syntax#lists). Ruby’s
Array methods may be called in policies:

```polar
allow(actor, _action, _resource) if actor.groups.include?("HR");
```

```ruby
class User
  attr_reader :groups

  def initialize(groups)
    @groups = groups
  end
end

user = User.new(["HR", "payroll"])
oso.authorize(user, "foo", "bar")
```

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate lists in place unless the list is
also returned from the method.
{{% /callout %}}

Likewise, lists constructed in Polar may be passed into Ruby methods:

```polar
allow(actor, _action, _resource) if actor.has_groups?(["HR", "payroll"]);
```

```ruby
class User
  attr_reader :groups

  def initialize(groups)
    @groups = groups
  end

  def has_groups(other)
    @groups & other == other
  end
end

user = User.new(["HR", "payroll"])
oso.authorize(user, "foo", "bar")
```

Ruby methods like [`Array#at`][rb-array-at] may be used for random access to
list elements, but there is currently no Polar syntax that is equivalent to the
Ruby expression `user.groups[1]`. To access the elements of a list without
using a method, you may iterate over it with [the `in`
operator](polar-syntax#in-list-membership) or destructure it with [pattern
matching](polar-syntax#patterns-and-matching).

### Hashes

Ruby hashes are mapped to Polar [dictionaries](polar-syntax#dictionaries):

```polar
allow(actor, _action, _resource) if actor.roles.project1 = "admin";
```

```ruby
class User
  attr_reader :roles

  def initialize(roles)
    @roles = roles
  end
end

user = User.new({"project1" => "admin"})
oso.authorize(user, "foo", "bar")
```

Likewise, dictionaries constructed in Polar may be passed into Ruby methods.

### Enumerables

You may iterate over any Ruby [enumerable][rb-enumerable] using Polar's [`in`
operator](polar-syntax#in-list-membership):

```polar
allow(actor, _action, _resource) if "payroll" in actor.get_groups();
```

```ruby
class User
  def get_groups(self)
    ["HR", "payroll"]
  end
end

oso.authorize(User.new, "foo", "bar")
```

### `nil`

The Ruby value `nil` is registered as the Polar constant
[`nil`](reference/polar/polar-syntax#nil). If a Ruby method can return
`nil`, you may want to compare the result to Polar's `nil` in your policy:

```polar
allow(actor, _action, _resource) if actor.get_optional != nil;
```

```ruby
class User
  def get_optional
    some_condition? ? some_thing : nil
  end
end

oso.authorize(User.new, "foo", "bar")
```

### Ruby → Polar Types Summary

| Ruby type    | Polar type   |
| ------------ | ------------ |
| `Integer`    | `Integer`    |
| `Float`      | `Float`      |
| `TrueClass`  | `Boolean`    |
| `FalseClass` | `Boolean`    |
| `Array`      | `List`       |
| `Hash`       | `Dictionary` |
| `String`     | `String`     |
