---
instance: instance
isAdmin: IsAdmin
isAdminOf: IsAdminOf
isAllowed: IsAllowed
langName: Go
startswith: StartsWith
userParams: "\"alice\", true"

classMethodExample: |
  ```go
  type User string

  func (u User) Superusers() []User {
      return []{User("alice"), User("bhavik"), User("clarice")}
  }

  oso.RegisterClass(User)
  assert.True(oso.IsAllowed(user, "foo", "bar"))
  ```

registerClass: |
  We can register a go type using `Oso.RegisterClass()`

  ```go
  oso.RegisterClass(User)
  ```

specializedExample: |
  ```go
  oso.RegisterClass(User)

  user := User{"alice", true}
  assert.True(oso.IsAllowed(user, "foo", "bar"))
  assert.False(oso.IsAllowed("notauser", "foo", "bar"))
  ```

testQueries: |
  ```polar
  ?= allow(new User("alice", true), "foo", "bar");
  ```

userClass: |
  ```go
  type User struct {
      Name string
      IsAdmin bool
  }

  user := User{"alice", true}
  assert.True(oso.IsAllowed(user, "foo", "bar"))
  ```
---
