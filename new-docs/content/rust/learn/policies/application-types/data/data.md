---
instance: struct
isAdmin: is_admin
isAdminOf: is_admin_of
isAllowed: is_allowed
langName: Rust
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
