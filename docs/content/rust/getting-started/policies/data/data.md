---
instance: struct
isAdmin: is_admin
isAdminOf: is_admin_of
isAllowed: is_allowed
langName: Rust
startswith: starts_with
userParams: "\"alice\", true"

classMethodExample: |
  ```rust
  #[derive(Clone, PolarClass)]
  struct User {
      #[polar(attribute)]
      name: String,
  }

  impl User {
      fn superusers() -> Vec<String> {
          vec![
              "alice".to_string(),
              "bhavik".to_string(),
              "clarice".to_string(),
          ]
      }
  }

  oso.register_class(
      User::get_polar_class_builder()
          .add_class_method("superusers". User::superusers)
          .build(),
  )?;

  let user = User { name: "alice".to_string() };
  assert!(oso.is_allowed(user, "foo", "bar)?);
  ```

registerClass: |
  We can register a Rust `struct` or `enum` using `Oso::register_class()`.
  `register_class`() takes as input a `Class`, which can be constructed either
  using the `#[derive(PolarClass)]` procedural macro, or manually using
  `Class::new::<T>()`:

  ```rust
  #[derive(Clone, PolarClass)]
  struct User {
      #[polar(attribute)]
      name: String,
      #[polar(attribute)]
      is_admin: bool,
  }

  impl User {
      fn new(name: String, is_admin: bool) -> Self {
          Self { name, is_admin }
      }

      fn is_called_alice(&self) -> bool {
          self.name == "alice"
      }
  }

  oso.register_class(
     User::get_polar_class_builder()
          .set_constructor(User::new)
          .add_method("is_called_alice", User::is_called_alice)
          .build(),
  )?;
  ```

specializedExample: |
  ```rust
  #[derive(Clone, PolarClass)]
  struct User {
      #[polar(attribute)]
      name: String,
      #[polar(attribute)]
      is_admin: bool,
  }
  oso.register_class(User::get_polar_class())?;

  let user = User { name: "alice".to_string(), is_admin: true };
  assert!(oso.is_allowed(user, "foo", "bar")?);
  assert!(!oso.is_allowed("notauser", "foo", "bar")?);
  ```

testQueries: |
  ```polar
  ?= allow(new User("bob", true), "foo", "bar");
  ?= new User("alice", true).is_called_alice();
  ```

userClass: |
  ```rust
  #[derive(Clone, PolarClass)]
  struct User {
      #[polar(attribute)]
      name: String,
      #[polar(attribute)]
      is_admin: bool,
  }
  oso.register_class(User::get_polar_class())?;
  let user = User { name: "alice".to_string(), is_admin: true };
  assert!(oso.is_allowed(user, "foo", "bar")?);
  ```
---
