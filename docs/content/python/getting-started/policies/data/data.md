---
instance: instance
isAdmin: is_admin
isAdminOf: is_admin_of
isAllowed: is_allowed
langName: Python
startswith: startswith
userParams: "\"alice\", true"

classMethodExample: |
  ```python
  class User:
      ...
      @classmethod
      def superusers(cls):
          """ Class method to return list of superusers. """
          return ["alice", "bhavik", "clarice"]

  oso.register_class(User)

  user = User("alice", True)
  assert(oso.is_allowed(user, "foo", "bar))
  ```

registerClass: |
  We can register a Python class using `Oso.register_class()` or the
  `Oso.polar_class()` decorator:

  ```polar
  oso.register_class(User)
  ```

specializedExample: |
  ```python
  oso.register_class(User)

  user = User("alice", True)
  assert oso.is_allowed(user, "foo", "bar")
  assert not oso.is_allowed("notauser", "foo", "bar")
  ```

testQueries: |
  ```polar
  ?= allow(new User("alice", true), "foo", "bar");
  ```

userClass: |
  ```python
  class User:
      def __init__(self, name, is_admin):
          self.name = name
          self.is_admin = is_admin

  user = User("alice", True)
  assert(oso.is_allowed(user, "foo", "bar"))
  ```
---
