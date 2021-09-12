---
app_path: examples/rbac/nodejs/app.js
authorize: |
    ```js
    await oso.authorize(new User("Ariana"), "push", new Repository("Acme App"));
    ```
---
