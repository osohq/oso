---
title: "Implement Resource Hierarchies"
weight: 3
showContentForAnyLanguage: true
---

# How Implement Resource Hierarchies

A **resource hierarchy** refers to a model with nested resources, where a user's permissions and roles on a resource depend on the resource's parent.

Common examples of resource hierarchies include:
- File system permissions: access to a folder may grant access to documents within the folder
- Grouping resources by project: users have project-level roles and permissions that determine their access to resources within the project
- Organizations and multi-tenancy: top-level tenant/organization roles and permissions grant access to resources within the organization


<!-- TODO: something about how we use relationships to model resource hierarchies? -->
You can model resource hierarchies in Oso by defining *relationships* between resources.
You can write policies that use relationships and query them to find out if a user has access to a single resource or to get a list of resources that a user has access to (using our data filtering feature).

This guide uses an example resource hierarchy from our GitClub sample application.
In GitClub, `Organization` is the top-level resource, and `Repository` resources are nested within organizations.
Users have roles at both the organization and repository level. A user's organization role grants them a default role on every repository within that organization.

## 1. Register resource types and relationships

The first step to modeling a resource hierarchy is to register the application types that represent the resources you are protecting.

To make your implementation compatible with [data filtering](link TODO), you need to specify resource relationships by creating `Relationship` objects and passing them to `register_class()`. For more information on registering classes for data filtering, see the [data filtering guide](link TODO).

<!-- GitClub has three resource types (`Organization`, `Repository`, and `Issue`), which  -->

{{< code file="app.py" hl_lines="15,16,17,28,29,30" >}}
from polar import Relationship
from oso import Oso

oso = Oso()

# Register the Organization class
oso.register_class(Organization, types={"id": str}, fetcher=get_orgs)

# Register the Repository class, and its "parent" relationship to the Organization class
oso.register_class(
    Repository,
    types={
        "id": str,
        "org_id": str,
        "org": Relationship(
            kind="parent", other_type="Organization", my_field="org_id", other_field="id"
        ),
    },
    fetcher=get_repos,
)
{{< /code >}}

## 2. Declare parent relations

After registering your resource types, you can define a [resource block](reference/polar/polar-syntax#actor-and-resource-blocks) for each resource in your policy.

Inside each block, you should declare the permissions and roles that are available on that resource type.
**For child resource types, also [declare relations](reference/polar/polar-syntax#relation-declarations) to parent resources.**

{{< code file="main.polar" hl_lines="9, 14" >}}
resource Organization {
    permissions = ["read", "add_member"];
    roles = ["member", "owner"];
}

resource Repository {
    permissions = ["read", "push"];
    roles = ["contributor", "maintainer", "admin"];
    relations = {parent: Organization};
}
{{< /code >}}

Now that you've defined your resource relations in Polar, you can hook them up to the `Relationship`s you registered in Step 1 using `has_relation` rules:

{{< code file="main.polar" >}}
has_relation(parent_org: Organization, "parent", child_repo: Repository) if
    parent_org = child_repo.org;    # use the `org` relationship we registered in Step 1
{{< /code >}}

{{% callout "Tip" green %}}
Using registered `Relationship`s to access related resources from your policy, rather than using an arbitrary application field or method, ensures that data filtering queries will work with your policy.
{{% /callout %}}

## 3. Write rules using parent relations

You now have all the plumbing you need in place to write rules that use `parent` relations.

If you need to grant a role on a child resource based on a parent resource role, you can define a [shorthand rule](reference/polar/polar-syntax#shorthand-rules) in the child resource block. For example, in GitClub the `"owner"` role on `Organization` resources grants a user the `"admin"` role on every `Repository` within the organization:

{{< code file="main.polar" hl_lines="11" >}}
resource Organization {
    permissions = ["read", "add_member"];
    roles = ["member", "owner"];
}

resource Repository {
    permissions = ["read", "push"];
    roles = ["contributor", "maintainer", "admin"];
    relations = {parent: Organization};

    "admin" if "owner" on "parent";
}
{{< /code >}}

You can also use shorthand rules to grant permissions based on parent resource roles and permissions.
For example, we could add that users with the Organization `"member"` role can `"read"` every repository in the organization:


{{< code file="main.polar" hl_lines="12" >}}
resource Repository {
    # ...
    "admin" if "owner" on "parent";
    "read" if "member" on "parent";
}
{{< /code >}}

We could also modify that rule to say that users who have the Organization `"read"` permission have the `"read"` permission on every repository in the organization.

{{< code file="main.polar" hl_lines="12" >}}
resource Repository {
    # ...
    "admin" if "owner" on "parent";
    "read" if "read" on "parent";
}
{{< /code >}}
