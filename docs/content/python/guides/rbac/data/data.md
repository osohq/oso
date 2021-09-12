---
app_path: examples/rbac/python/app.py
authorize: |
    ```py
    oso.authorize(User("Ariana"), "push", Repository("Acme App"))
    ```
---
