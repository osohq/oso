---
title: Application Data
---

## Working with Rust Types

oso’s Rust authorization library allows you to write policy rules over Rust types directly.
This document explains how different Rust types can be used in oso policies.

**NOTE**: More detailed examples of working with application objects can be found in Policy Examples.

### Structs + Enums

Rust structs and enums can be registered with oso which lets you pass them in and access their methods and fields. (see Application Types).

Rust structs can also be constructed from inside an oso policy using the New operator if the type has been given a constructor when registered.

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans (see Primitive Types).

### Strings

Rust [Strings](https://doc.rust-lang.org/std/string/struct.String.html) are mapped to Polar Strings. Many of rust’s string methods may be called in policies:

```
allow(actor, action, resource) if actor.username.ends_with("example.com");
```

```
#[derive(Clone, PolarClass)]
struct User {
  #[polar(attribute)]
  pub username: String
}

oso.register_class(User::get_polar_class())?;

let user = User{username: "alice@example.com".to_owned()};
assert!(oso.is_allowed(user, "foo", "bar")?);
```

**WARNING**: Polar does not support methods that mutate strings in place.

### Vectors

[Vec<T>](https://doc.rust-lang.org/std/vec/struct.Vec.html) is mapped to Polar Lists, given that `T: ToPolar`.

Currently, no methods on `Vec` are exposed to Polar.

```
allow(actor, action, resource) if "HR" in actor.groups;
```

```
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    pub groups: Vec<String>,
}

oso.register_class(User::get_polar_class())?;

let user = User { groups: vec!["HR".to_string(), "payroll".to_string()] };
assert!(oso.is_allowed(user, "foo", "bar")?);
```

**WARNING**: Polar does not support methods that mutate lists in place, unless the list is also returned from the method.

Rust methods like [`Vec::get`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.get) may be used for random access to
list elements, but there is currently no Polar syntax that is
equivalent to the Rust expression `user.groups[1]`. To access
the elements of a list without using a method, you may iterate
over it with In (List Membership) or destructure it with
pattern matching.

### HashMaps

Rust [HashMaps](https://doc.rust-lang.org/std/collections/struct.HashMap.html) are mapped to Polar Dictionaries,
but require that the `HashMap` key is a `String`:

```
allow(actor, action, resource) if actor.roles.project1 = "admin";
```

```
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    pub roles: HashMap<String, String>,
}

oso.register_class(User::get_polar_class())?;

let user = User { roles: maplit::hashmap!{ "project1".to_string() => "admin".to_string() } };
assert!(oso.is_allowed(user, "foo", "bar")?);
```

Likewise, dictionaries constructed in Polar may be passed into Ruby methods.

### Iterators

You may iterate over a Rust [iterator](https://doc.rust-lang.org/std/iter/index.html)
using the Polar In (List Membership) operator:

```
allow(actor, action, resource) if "payroll" in actor.get_groups();
```

```
  #[derive(Clone, PolarClass)]
  struct User {
      groups: Vec<String>,
  }

  oso.register_class(
      User::get_polar_class_builder()
          .add_iterator_method("get_groups", |u: &User| u.groups.clone().into_iter())
          .build(),
  )
  .unwrap();

  let user = User {
      groups: vec!["HR".to_string(), "payroll".to_string()],
  };
  assert!(oso.is_allowed(user, "foo", "bar")?);
```

### Options

The Rust type `Option<T>` is registered as a class.
You can use `unwrap` on an option in a policy, but the safer way
is to use the `in` operator, which will return 0 or 1 values depending
on if the value is `None` or `Some(_)` respectively.

The value `None` is registered as the Polar constant nil.
If a Rust method can return `None`, you may want to compare the result
to `nil`:

```
allow(actor, action, resource) if "Jimmy" in actor.nickname or actor.get_optional() != nil;
```

```
  #[derive(Clone, PolarClass)]
  struct User {
      #[polar(attr)]
      nickname: Option<String>,
  }

  oso.register_class(
      User::get_polar_class_builder()
          .add_method("get_optional", |u: &User| None)
          .build(),
  )
  .unwrap();

  let user = User { nickname: Some("Jimmy".to_string()), };
  assert!(oso.is_allowed(user, "foo", "bar")?);
```

### Summary

### Rust → Polar Types Summary

| Rust type

 | Polar type

 |     |
 | --- ||  |  |  |  |  ||  |  |  |  |  |  |  |  |  |  |  |
| i32, i64, usize

                                                | Integer

                                                                                 |
| f32, f64

                                                       | Float

                                                                                   |
| bool

                                                           | Boolean

                                                                                 |
| Vec

                                                            | List

                                                                                    |
| HashMap

                                                        | Dictionary

                                                                              |
| String, &’static str, str

                                      | String

                                                                                  |
