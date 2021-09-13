---
app_path: examples/rbac/python/app.py
policy_path: examples/rbac/python/main.polar
authorize: |
    ```py
    oso.authorize(User("Ariana"), "push", Repository("Acme App"))
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
