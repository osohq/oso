---
langName: Java
userClassPath: examples/rbac/java/User.java
registeredUserClass: |
    import com.osohq.oso.*;

    public class User {
      ...

      public static void main(String[] args) {
        Oso oso = Oso();
        oso.registerClass(User.class);
      }
    }
---
