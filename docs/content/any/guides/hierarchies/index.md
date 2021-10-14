---
title: "Build Authorization for Resource Hierarchies"
weight: 4
showContentForAnyLanguage: true
---

# Build Authorization for Resource Hierarchies

A **resource hierarchy** refers to a model with nested resources, where a user's permissions and roles on a resource depend on the resource's parent.

Common examples of resource hierarchies include:
- File system permissions: access to a folder may grant access to documents within the folder
- Grouping resources by project: users have project-level roles and permissions that determine their access to resources within the project
- Organizations and multi-tenancy: top-level tenant/organization roles and permissions grant access to resources within the organization


You can model resource hierarchies in Oso by defining *relations* between resources.
You can write policies that use relations and query them to find out if a user has access to a single resource or to get a list of resources that a user has access to (using the [data filtering](guides/data_filtering) feature).

This guide uses an example resource hierarchy from our [GitClub][] sample application.
In GitClub, `Organization` is the top-level resource, and `Repository` resources are nested within organizations.
Users have roles at both the organization and repository level. A user's organization role grants them a default role on every repository within that organization.

[GitClub]: https://github.com/osohq/gitclub

## 1. Register resource types and relations

The first step to modeling a resource hierarchy is to register the application types that represent the resources you are protecting.

To make your implementation compatible with [data filtering](guides/data_filtering), you need to specify resource relations by creating `Relation` objects and passing them to `register_class()`. For more information on registering classes and relations for data filtering, see the [data filtering guide](guides/data_filtering#relations).

<!-- GitClub has three resource types (`Organization`, `Repository`, and `Issue`), which  -->
{{< literalInclude dynPath=registerClassExample
                   fallback="no" >}}

## 2. Declare parent relations

After registering your resource types, you can define a [resource block](reference/polar/polar-syntax#actor-and-resource-blocks) for each resource in your policy.

Inside each block, you should declare the permissions and roles that are available on that resource type.
**For child resource types, also [declare relations](reference/polar/polar-syntax#relation-declarations) to parent resources.**

{{< code file="main.polar" hl_lines="11, 16" >}}
allow(actor, action, resource) if has_permission(actor, action, resource);

resource Organization {
    permissions = ["read", "add_member"];
    roles = ["member", "owner"];
}

resource Repository {
    permissions = ["read", "push"];
    roles = ["contributor", "maintainer", "admin"];
    relations = { parent: Organization };
}
{{< /code >}}

Now that you've defined your resource relations in Polar, you can hook them up to the `Relation`s you registered in Step 1 using `has_relation` rules:

{{< code file="main.polar" >}}
has_relation(parent_org: Organization, "parent", child_repo: Repository) if
    parent_org = child_repo.organization;    # use the `organization` relation we registered in Step 1
{{< /code >}}

{{% callout "Tip" "green" %}}
Using registered `Relation`s to access related resources from your policy, rather than using an arbitrary application field or method, ensures that data filtering queries will work with your policy.
{{% /callout %}}

## 3. Write rules using parent relations

You now have all the plumbing in place to write rules that use `parent` relations.

If you need to grant a role on a child resource based on a parent resource role, you can define a [shorthand rule](reference/polar/polar-syntax#shorthand-rules) in the child resource block. For example, in GitClub the `"owner"` role on `Organization` resources grants a user the `"admin"` role on every `Repository` within the organization:

{{< code file="main.polar" hl_lines="13" >}}
allow(actor, action, resource) if has_permission(actor, action, resource);

resource Organization {
    permissions = ["read", "add_member"];
    roles = ["member", "owner"];
}

resource Repository {
    permissions = ["read", "push"];
    roles = ["contributor", "maintainer", "admin"];
    relations = { parent: Organization };

    "admin" if "owner" on "parent";
}
{{< /code >}}

You can also use shorthand rules to grant permissions based on parent resource roles and permissions.
For example, we could add that users with the Organization `"member"` role can `"read"` every repository in the organization:


{{< code file="main.polar" hl_lines="13" >}}
# ...
resource Repository {
    # ...
    "admin" if "owner" on "parent";
    "read" if "member" on "parent";
}
{{< /code >}}

We could also modify that rule to say that users who have the Organization `"read"` permission have the `"read"` permission on every repository in the organization.

{{< code file="main.polar" hl_lines="13" >}}
# ...
resource Repository {
    # ...
    "admin" if "owner" on "parent";
    "read" if "read" on "parent";
}
{{< /code >}}
