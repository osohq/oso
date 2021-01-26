---
instance: instance
isAdmin: is_admin
isAdminOf: is_admin_of
isAllowed: is_allowed
langName: Python
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
