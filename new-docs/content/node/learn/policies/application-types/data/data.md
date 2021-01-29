---
instance: object
isAdmin: isAdmin
isAdminOf: isAdminOf
isAllowed: isAllowed
langName: JavaScript
startswith: startsWith
userParams: "\"alice\", true"

classMethodExample: |
  ```js
  class User {
    constructor (name, isAdmin) {
      this.name = name;
      this.isAdmin = isAdmin;
    }

    static superusers() {
      return ['alice', 'bhavik', 'clarice'];
    }
  }

  oso.registerClass(User);
  const user = new User('alice', true);

  (async () => assert(await oso.isAllowed(user, "foo", "bar")))();
  ```

registerClass: |
  JavaScript classes are registered using `registerClass()`:

  ```js
  oso.registerClass(User);
  ```

specializedExample: |
  ```js
  oso.registerClass(User);
  const user = new User('alice', true);

  (async () => {
    assert.equal(true, await oso.isAllowed(user, "foo", "bar"));
    assert.equal(false, await oso.isAllowed("notauser", "foo", "bar"));
  })();
  ```

testQueries: |
  ```polar
  ?= allow(new User("alice", true), "foo", "bar");
  ```

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
