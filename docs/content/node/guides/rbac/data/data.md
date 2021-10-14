---
app_path: examples/rbac/nodejs/app.js
policy_path: examples/rbac/nodejs/main.polar
authorize: |
    ```js
    await oso.authorize(new User("Ariana"), "push", new Repository("Acme App"));
    ```
repository_roles: repositoryRoles
organization_roles: organizationRoles
role_name: name
role_organization: organization
role_repository: repository
roles: roles
role_resource: resource
authorize_method_name: authorize
---
