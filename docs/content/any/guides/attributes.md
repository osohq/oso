---
title: "Build Attribute-based Access Control (ABAC)"
weight: 1
---

# How to Build Attribute-based Access Control

While role-based access control emphasizes granting permissions based on roles, you may also wish to grant permissions or roles based on user or resource attributes. **With Oso, you can use attribute-based logic alongside roles.**

## Grant permissions with attributes

Granting users permissions based on attributes is simple with Oso. Let's say your policy contains the following [resource block](reference/polar/polar-syntax#actor-and-resource-blocks):

{{< code file="main.polar" hl_lines="10, 15" >}}
# ...
resource Repository {
	permissions = ["read", "push", "delete"];
	roles = ["contributor", "maintainer", "admin"];

    "read" if "contributor";
}
{{< /code >}}

The block contains a role-based rule that grants the `"read"` permission to actors who have the `"contributor"` role.

You can add an attribute-based rule that grants all users the `"read"` permission for any repository that is public:

{{< code file="main.polar" hl_lines="8" >}}
# ...
resource Repository {
	permissions = ["read", "push", "delete"];
	roles = ["contributor", "maintainer", "admin"];

    "read" if "contributor";
}
has_permission(_: User, "read", repo: Repository) if repo.is_public;
{{< /code >}}

The `has_permission` rule above tells Oso to look up the `is_public` attribute on the `Repository` application type in order to determine whether or not someone shoud be granted `"read"` access.
This rule will be evaluated alongside the `"read" if "contributor"` shorthand rule in the resource block, so that a user can read a repository if they have the `"contributor"` role OR if the repository is public.

## Grant roles with attributes

Oso also supports granting users roles based on user or resource attributes. Oso uses `has_role` rules to look up a user's roles in your application. By defining multiple `has_role` rules,  you can customize how users are granted various roles.

For example, you could add a `has_role` rule to the policy above that grants the `"admin"` role to the repository creator.

{{< code file="main.polar" hl_lines="8" >}}
# ...
resource Repository {
	permissions = ["read", "push", "delete"];
	roles = ["contributor", "maintainer", "admin"];

    "read" if "contributor";
}
has_role(user: User, "read", repo: Repository) if user = repo.created_by;
{{< /code >}}


