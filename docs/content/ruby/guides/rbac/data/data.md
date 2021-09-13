---
app_path: examples/rbac/ruby/app.rb
policy_path: examples/rbac/ruby/main.polar
authorize: |
    ```rb
    oso.authorize(User.new("Ariana"), "push", Repository.new("Acme App"))
    ```
---
