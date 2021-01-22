---
title: Application Data
---

## Working with Ruby Objects

oso’s Ruby authorization library allows you to write policy rules over Ruby objects directly.
This document explains how different types of Ruby objects can be used in oso policies.

**NOTE**: More detailed examples of working with application objects can be found in Policy Examples.

### Class Instances

You can pass any Ruby instance into oso and access its methods and fields from your policy (see Application Types).

Ruby instances can be constructed from inside an oso policy using the New operator if the Ruby class has been **registered** using
the `#register_class` method. An example of this can be found here.

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans (see Primitive Types).
These map to the Ruby `Integer`, `Float`, and `TrueClass`/`FalseClass` types.

### Strings

Ruby strings are mapped to Polar Strings. Ruby’s string methods may be called in policies:

```
allow(actor, action, resource) if actor.username.end_with?("example.com");
```

```
class User
  attr_reader :username

  def initialize(username)
    @username = username
  end
end

user = User.new("alice@example.com")
raise "should be allowed" unless oso.allowed?(user, "foo", "bar")
```

**WARNING**: Polar does not support methods that mutate strings in place.

### Lists

Ruby [Arrays](https://ruby-doc.org/core/Array.html) are mapped to Polar Lists. Ruby’s Array methods may be called in policies:

```
allow(actor, action, resource) if actor.groups.include?("HR");
```

```
class User
  attr_reader :groups

  def initialize(groups)
    @groups = groups
  end
end

user = User.new(["HR", "payroll"])
raise "should be allowed" unless oso.allowed?(user, "foo", "bar")
```

**WARNING**: Polar does not support methods that mutate lists in place, unless the list is also returned from the method.

Likewise, lists constructed in Polar may be passed into Ruby methods:

```
allow(actor, action, resource) if actor.has_groups?(["HR", "payroll"]);
```

```
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
raise "should be allowed" unless oso.allowed?(user, "foo", "bar")
```

Ruby methods like [`Array#at`](https://ruby-doc.org/core/Array.html#method-i-at) may be used for random access to
list elements, but there is currently no Polar syntax that is
equivalent to the Ruby expression `user.groups[1]`. To access
the elements of a list without using a method, you may iterate
over it with In (List Membership) or destructure it with
pattern matching.

### Hashes

Ruby hashes are mapped to Polar Dictionaries:

```
allow(actor, action, resource) if actor.roles.project1 = "admin";
```

```
class User
  attr_reader :roles

  def initialize(roles)
    @roles = roles
  end
end

user = User.new({"project1" => "admin"})
raise "should be allowed" unless oso.allowed?(user, "foo", "bar")
```

Likewise, dictionaries constructed in Polar may be passed into Ruby methods.

### Enumerables

You may iterate over any Ruby [enumerable](https://ruby-doc.org/core/Enumerable.html)
using the Polar In (List Membership) operator:

```
allow(actor, action, resource) if "payroll" in actor.get_groups();
```

```
class User
  def get_groups(self)
    ["HR", "payroll"]
  end
end

user = User.new
raise "should be allowed" unless oso.allowed?(user, "foo", "bar")
```

### `nil`

The Ruby value `nil` is registered as the Polar constant nil.
If a Ruby method can return `None`, you may want to compare the result
to `nil`:

```
allow(actor, action, resource) if actor.get_optional? != nil;
```

```
class User
   def get_optional(self)
      if some_condition?
        some_thing
      else
        nil
   end
end

user = User.new
raise "should be allowed" unless oso.allowed?(user, "foo", "bar")
```

<!-- ### Summary

### Ruby → Polar Types Summary

| Ruby type

 | Polar type

 |     |
 | --- ||  |  |  |  |  ||  |  |  |  |  |  |  |  |  |
| Integer

                                                        | Integer

                                                                                 |
| Float

                                                          | Float

                                                                                   |
| TrueClass

                                                      | Boolean

                                                                                 |
| FalseClass

                                                     | Boolean

                                                                                 |
| Array

                                                          | List

                                                                                    |
| Hash

                                                           | Dictionary

                                                                              |
| String

                                                         | String

                                                                                  | -->
