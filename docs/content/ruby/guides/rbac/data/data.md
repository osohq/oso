---
app_path: examples/rbac/ruby/app.rb
policy_path: examples/rbac/ruby/main.polar
authorize: |
    ```rb
    oso.authorize(User.new("Ariana"), "push", Repository.new("Acme App"))
    ```
repository_roles: repository_roles
organization_roles: organization_roles
role_name: name
role_organization: organization
role_repository: repository
roles: roles
role_resource: resource
authorize_method_name: authorize
---
