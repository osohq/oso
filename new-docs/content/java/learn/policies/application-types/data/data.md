---
instance: instance
isAdmin: isAdmin
isAdminOf: isAdminOf
isAllowed: isAllowed
langName: Java
userClass: |
  ```java
  public class User {
      public boolean isAdmin;
      public String name;

      public User(String name, boolean isAdmin) {
          this.isAdmin = isAdmin;
          this.name = name;
      }

      public static void main(String[] args) {
          User user = new User("alice", true);
          assert oso.isAllowed(user, "foo", "bar");
      }
  }
  ```
---
