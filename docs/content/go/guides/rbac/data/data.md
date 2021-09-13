---
app_path: examples/rbac/go/app.go
authorize: |
    ```go
    oso.Authorize(User{Name: "Ariana"}, "push", Repository{Name: "Acme App"})
    ```
---
