---
app_path: examples/rbac/python/app.py
policy_path: examples/rbac/python/main.polar
authorize: |
    ```py
    oso.authorize(User("Ariana"), "push", Repository("Acme App"))
    ```
---
