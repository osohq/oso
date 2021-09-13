---
app_path: examples/rbac/ruby/app.rb
authorize: |
    ```rb
    oso.authorize(User.new("Ariana"), "push", Repository.new("Acme App"))
    ```
---
