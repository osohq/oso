---
instance: instance
isAdmin: isAdmin
isAdminOf: isAdminOf
isAllowed: isAllowed
langName: Java
startswith: startsWith
userParams: "\"alice\", true"

classMethodExample: |
  ```java
  public static List<String> superusers() {
      return List.of("alice", "bhavik", "clarice");
  }

  public static void main(String[] args) {
      oso.registerClass(User.class);

      User user = new User("alice", true);
      assert oso.isAllowed(user, "foo", "bar");
  }
  ```

registerClass: |
  Java classes are registered using `registerClass()`:

  ```java
  public static void main(String[] args) {
      oso.registerClass(User.class);
  }
  ```

  You may register a Java class with a particular
  [Constructor](https://docs.oracle.com/en/java/javase/11/docs/api/java.base/java/lang/reflect/Constructor.html),
  but the default behavior is to choose one at instantiation time based on the
  classes of the supplied arguments. For the example above, this would probably
  be a constructor with a signature like `public User(String name, bool
  isAdmin)`. See [the Java library
  documentation](reference/polar/classes#class-instances) for more details.

specializedExample: |
  ```java
  public static void main(String[] args) {
      oso.registerClass(User.class);

      User user = new User("alice", true);
      assert oso.isAllowed(user, "foo", "bar");
      assert !oso.isAllowed("notauser", "foo", "bar");
  }
  ```

testQueries: |
  ```polar
  ?= allow(new User("alice", true), "foo", "bar");
  ```

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
