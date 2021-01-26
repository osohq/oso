---
instance: object
isAdmin: isAdmin
isAdminOf: isAdminOf
isAllowed: isAllowed
langName: JavaScript
userClass: |
  ```js
  class User {
    constructor (name, isAdmin) {
      this.name = name;
      this.isAdmin = isAdmin;
    }
  }

  const user = new User("alice", true);

  (async () => {
    const decision = await oso.isAllowed(user, 'foo', 'bar');
    assert(decision);
  })();
  ```
---
