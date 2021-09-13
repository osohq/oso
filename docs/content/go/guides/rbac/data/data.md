---
app_path: examples/rbac/go/app.go
policy_path: examples/rbac/go/main.polar
authorize: |
    ```go
    oso.Authorize(User{Name: "Ariana"}, "push", Repository{Name: "Acme App"})
    ```
---
