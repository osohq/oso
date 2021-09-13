---
app_path: examples/rbac/java/App.java
policy_path: examples/rbac/java/main.polar
authorize: |
    ```java
    oso.authorize(new User("Ariana"), "push", new Repository("Acme App"));
    ```
---
