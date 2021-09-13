---
app_path: examples/rbac/java/App.java
policy_path: examples/rbac/java/main.polar
authorize: |
    ```java
    oso.authorize(new User("Ariana"), "push", new Repository("Acme App"));
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
