---
title: Rust Types in Polar
weight: 2
aliases:
  - /using/libraries/rust/index.html
description: |
    Reference for using Rust types in Polar.
---

[rust-string]: https://doc.rust-lang.org/std/string/struct.String.html
[rust-vec]: https://doc.rust-lang.org/std/vec/struct.Vec.html
[rust-vec-get]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.get
[rust-hashmap]: https://doc.rust-lang.org/std/collections/struct.HashMap.html

## Working with Rust Types

Oso’s Rust authorization library allows you to write policy rules over Rust
types directly. This document explains how different Rust types can be used in
Oso policies.

{{% callout "Note" "blue" %}}
More detailed examples of working with application objects can be found in our
[Guides](guides).
{{% /callout %}}

### Structs + Enums

Rust structs and enums can be registered with Oso, which lets you pass them in
and access their methods and fields in your policy (see [Application
Types](guides/policies#instances-and-fields)).

Rust structs can also be constructed from inside an Oso policy using [the `new`
operator](polar-syntax#new) if a type constructor is provided at registration.

### Numbers and Booleans

Polar supports integer and floating point real numbers, as well as booleans
(see [Primitive Types](polar-syntax#primitive-types)).

### Strings

Rust [Strings][rust-string] are mapped to Polar
[strings](polar-syntax#strings). Many of Rust’s String methods may be called in
policies:

```polar
allow(actor, _action, _resource) if actor.username.ends_with("example.com");
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

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate strings in place.
{{% /callout %}}

### `Vec`

[`Vec<T>`][rust-vec] maps to a Polar [list](polar-syntax#lists), given that `T: ToPolar`.

Implementations also exist to convert `LinkedList`, `VecDeque`,
`BinaryHeap`, `HashSet`, and `BTreeSet`to and from Polar lists,
but lists are treated as `Vec<T>` when calling methods.

Currently, no methods on `Vec` are exposed to Polar.

```polar
allow(actor, _action, _resource) if "HR" in actor.groups;
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

{{% callout "Warning" "orange" %}}
Polar does not support methods that mutate lists in place unless the list is
also returned from the method.
{{% /callout %}}

Rust methods like [`Vec::get`][rust-vec-get] may be used for random access to
list elements, but there is currently no Polar syntax that is equivalent to the
Rust expression `user.groups[1]`. To access the elements of a list without
using a method, you may iterate over it with [the `in`
operator](polar-syntax#in-list-membership) or destructure it with [pattern
matching](polar-syntax#patterns-and-matching).

### `HashMap`

A Rust [`HashMap`][rust-hashmap] maps to a Polar
[dictionary](polar-syntax#dictionaries) but requires that the `HashMap` key is
a `String`.

Implementations also exist to convert `BTreeMap`s to and
from Polar dictionaries, but dictionaries are treated as `HashMap` when calling methods.


```polar
allow(actor, _action, _resource) if actor.roles.project1 = "admin";
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
allow(actor, _action, _resource) if "payroll" in actor.get_groups();
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
[`nil`](reference/polar/polar-syntax#nil). If a Rust method can return
`None`, you may want to compare the result to `nil`:

```polar
allow(actor, _action, _resource) if
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

### UUIDs via the `uuid` crate

Oso supports UUIDs via the [`uuid`](https://crates.io/crates/uuid) crate behind
a feature flag. To enable support, you'll need to add a feature flag to your
`Cargo.toml` file and make sure you have the `uuid` crate as a separate
dependency. In `Cargo.toml`, an Oso dependency that supports UUIDs looks as
follows:

```toml
oso = { version = "X.Y.Z", features = ["uuid-10"] }
```

**Note that the numbers in the feature flags do not refer to [the UUID
version][wiki] but to the version of the `uuid` crate.** Most people will want
the `uuid-10` feature flag, as it supports recent versions of the `uuid` crate.

[wiki]: https://en.wikipedia.org/wiki/Universally_unique_identifier#Versions

| `uuid` Crate Version | Feature Flag |
|----------------------|--------------|
| `0.6.5` - `0.6.x`    | `uuid-06`    |
| `0.7.0` - `0.8.x`    | `uuid-07`    |
| `1.0.0` - `2.0.0`    | `uuid-10`    |

### Rust → Polar Types Summary

| Rust type                                                             | Polar type   |
| --------------------------------------------------------------------- | ------------ |
| `i32`, `i64`, `usize`                                                 | `Integer`    |
| `f32`, `f64`                                                          | `Float`      |
| `bool`                                                                | `Boolean`    |
| `String`, `&'static str`, `str`                                       | `String`     |
| `HashMap`, `BTreeMap`                                                 | `Dictionary` |
| `Vec`, `LinkedList`, `VecDeque` `BinaryHeap`, `HashSet`, `BTreeSet`   | `List`       |
| UUID (behind a feature flag)                                          | `Uuid`       |
