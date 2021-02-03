---
title: Rust Types in Polar
weight: 2
---

[rust-string]: https://doc.rust-lang.org/std/string/struct.String.html
[rust-vec]: https://doc.rust-lang.org/std/vec/struct.Vec.html
[rust-vec-get]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.get
[rust-hashmap]: https://doc.rust-lang.org/std/collections/struct.HashMap.html

## Working with Rust Types

oso’s Rust authorization library allows you to write policy rules over Rust
types directly. This document explains how different Rust types can be used in
oso policies.

{{< callout "Note" "blue" >}}
More detailed examples of working with application objects can be found in
[Policy Examples](learn/examples).
{{< /callout >}}

### Structs + Enums

Rust structs and enums can be registered with oso, which lets you pass them in
and access their methods and fields in your policy (see [Application
Types](learn/policies/application-types)).

Rust structs can also be constructed from inside an oso policy using [the `new`
operator](polar-syntax#new) if a type constructor is provided at registration.

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans
(see [Primitive Types](polar-syntax#primitive-types)).

### Strings

Rust [Strings][rust-string] are mapped to Polar
[strings](polar-syntax#strings). Many of Rust’s String methods may be called in
policies:

```polar
allow(actor, action, resource) if actor.username.ends_with("example.com");
```

```rust
#[derive(Clone, PolarClass)]
struct User {
  #[polar(attribute)]
  pub username: String
}

oso.register_class(User::get_polar_class())?;

let user = User{username: "alice@example.com".to_owned()};
assert!(oso.is_allowed(user, "foo", "bar")?);
```

{{< callout "Warning" "orange" >}}
Polar does not support methods that mutate strings in place.
{{< /callout >}}

### Vectors

[Vec\<T>][rust-vec] maps to a Polar [list](polar-syntax#lists), given that `T: ToPolar`.

Currently, no methods on `Vec` are exposed to Polar.

```polar
allow(actor, action, resource) if "HR" in actor.groups;
```

```rust
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    pub groups: Vec<String>,
}

oso.register_class(User::get_polar_class())?;

let user = User { groups: vec!["HR".to_string(), "payroll".to_string()] };
assert!(oso.is_allowed(user, "foo", "bar")?);
```

{{< callout "Warning" "orange" >}}
Polar does not support methods that mutate lists in place unless the list is
also returned from the method.
{{< /callout >}}

Rust methods like [`Vec::get`][rust-vec-get] may be used for random access to
list elements, but there is currently no Polar syntax that is equivalent to the
Rust expression `user.groups[1]`. To access the elements of a list without
using a method, you may iterate over it with [the `in`
operator](polar-syntax#in-list-membership) or destructure it with [pattern
matching](polar-syntax#patterns-and-matching).

### HashMaps

Rust [`HashMap`s][rust-hashmap] are mapped to Polar
[dictionaries](polar-syntax#dictionaries), but require that the `HashMap` key
is a `String`:

```polar
allow(actor, action, resource) if actor.roles.project1 = "admin";
```

```rust
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
    pub roles: HashMap<String, String>,
}

oso.register_class(User::get_polar_class())?;

let user = User { roles: maplit::hashmap!{
    "project1".to_string() => "admin".to_string()
}};
assert!(oso.is_allowed(user, "foo", "bar")?);
```

Likewise, dictionaries constructed in Polar may be passed into Rust methods.

### Iterators

You may iterate over a Rust
[iterator](https://doc.rust-lang.org/std/iter/index.html) using Polar's [`in`
operator](polar-syntax#in-list-membership):

```polar
allow(actor, action, resource) if "payroll" in actor.get_groups();
```

```rust
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

The Rust type `Option<T>` is registered as a class. You can use `unwrap()` on
an option in a policy, but it's safer to use the `in` operator, which will
return 0 or 1 values depending on whether the value is `None` or `Some(T)`
respectively.

The `Option` variant `None` is registered as the Polar constant
[`nil`](learn/policies/application-types#nil). If a Rust method can return
`None`, you may want to compare the result to `nil`:

```polar
allow(actor, action, resource) if
    "Jimmy" in actor.nickname or
    actor.get_optional() != nil;
```

```rust
#[derive(Clone, PolarClass)]
struct User {
    #[polar(attribute)]
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

### Rust → Polar Types Summary

| Rust type                 | Polar type |
| ------------------------- | ---------- |
| i32, i64, usize           | Integer    |
| f32, f64                  | Float      |
| bool                      | Boolean    |
| Vec                       | List       |
| HashMap                   | Dictionary |
| String, &’static str, str | String     |
