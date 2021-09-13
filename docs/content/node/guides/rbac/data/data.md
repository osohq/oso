---
app_path: examples/rbac/nodejs/app.js
policy_path: examples/rbac/nodejs/main.polar
authorize: |
    ```js
    await oso.authorize(new User("Ariana"), "push", new Repository("Acme App"));
    ```
---
