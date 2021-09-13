---
app_path: examples/rbac/java/App.java
authorize: |
    ```java
    oso.authorize(new User("Ariana"), "push", new Repository("Acme App"));
    ```
---
