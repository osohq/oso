---
app_path: examples/rbac/go/app.go
policy_path: examples/rbac/go/main.polar
authorize: |
    ```go
    oso.Authorize(User{Name: "Ariana"}, "push", Repository{Name: "Acme App"})
    ```
repository_roles: RepositoryRoles
organization_roles: OrganizationRoles
role_name: Name
role_organization: Organization
role_repository: Repository
roles: Roles
role_resource: Resource
authorize_method_name: Authorize
---
