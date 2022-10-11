---
title: Build Role-Based Access Control (RBAC)
metaTitle: Build Role-Based Access Control (RBAC) in $LANG with Oso
weight: -1
description: |
  Build role-based access control (RBAC) with Oso's built-in authorization
  modeling features.
aliases:
  - /guides/roles/index.html
  - /guides/roles/sqlalchemy/index.html
  - /getting-started/rbac.html
  - /guides/new-roles.html
---

# Build Role-Based Access Control (RBAC) in {{% lang %}} with Oso

Role-based access control (RBAC) is so ubiquitous that Oso provides
syntax for modeling RBAC. This syntax makes it easy to create a role-based
authorization policy with roles and permissions -- for example, declaring that
the `"maintainer"` role on a repository allows a user to `"push"` to that
repository. In this guide, we'll walk through building an RBAC policy for [GitClub][].

This guide assumes you're building authorization in a monolith application.
If you're building with microservices, read about [how to model roles using Oso Cloud](https://www.osohq.com/docs/guides/modeling-building-blocks).


<!--
{{% ifLang python %}}
To learn more about the fundamentals of Role-Based Access Control in
Python, visit our page on [RBAC Python][].
{{% /ifLang %}} -->

[RBAC Python]: https://www.osohq.com/learn/rbac-python
[GitClub]: https://github.com/osohq/gitclub

## Declare application types as actors and resources

Oso makes authorization decisions by determining if an **actor** can perform an
**action** on a **resource**:

- **Actor**: who is performing the action? `User("Ariana")`
- **Action**: what are they trying to do? `"push"`
- **Resource**: what are they doing it to? `Repository("Acme App")`

The first step of building an RBAC policy is telling Oso which application
types are **actors** and which are **resources**. Our example app has a pair of
**resource** types that we want to control access to, `Organization` and
`Repository`. We declare both as resources as follows:

<!-- TODO(gj): I guess these only need to be dedented when you use angle
bracket handlebars. -->
{{< code file="main.polar" >}}
resource Organization {}

resource Repository {}
{{< /code >}}

Our app also has a `User` type that will be our lone type of **actor**:

{{% literalInclude
  dynPath="policy_path"
  from="docs: begin-actor"
  to="docs: end-actor"
%}}

This piece of syntax is called a *resource block*, and it performs two
functions: it identifies the type as an **actor** or a **resource**, and it
provides a centralized place to declare roles and permissions for that
particular type.

{{% callout "Note" "blue" %}}
  For every resource block, we also need to register the type with Oso:

  {{< literalInclude
    dynPath="app_path"
    from="docs: begin-setup"
    to="docs: end-setup"
    hlFrom="docs: begin-register"
    hlTo="docs: end-register"
  >}}
{{% /callout %}}

## Declare roles and permissions

In GitClub, users can perform actions such as `"delete"`-ing an organization or
`"push"`-ing to a repository. Users can also be assigned roles for either type
of resource, such as the `"owner"` role for an `Organization` or the
`"maintainer"` role for a `Repository`.

Inside the curly braces of each `resource` block, we declare the roles and
permissions for that resource:

{{< code file="main.polar" >}}
resource Organization {
  roles = ["owner"];
}

resource Repository {
  permissions = ["read", "push"];
  roles = ["contributor", "maintainer"];
}
{{< /code >}}

<!-- TODO(gj): transition -->

## Grant permissions to roles

Next, we're going to write *shorthand rules* that grant permissions to roles.
For example, if we grant the `"push"` permission to the `"maintainer"` role in
the `Repository` resource block, then a user who's been assigned the
`"maintainer"` role for a particular repository can `"push"` to that
repository. Here's our `Repository` resource block with a few shorthand rules
added:

{{< code file="main.polar" hl_lines="5-10" >}}
resource Repository {
  permissions = ["read", "push"];
  roles = ["contributor", "maintainer"];

  # An actor has the "read" permission if they have the "contributor" role.
  "read" if "contributor";
  # An actor has the "read" permission if they have the "maintainer" role.
  "read" if "maintainer";
  # An actor has the "push" permission if they have the "maintainer" role.
  "push" if "maintainer";
}
{{< /code >}}

Shorthand rules expand to regular [Polar rules](polar-syntax#rules) when a
policy is loaded. The `"push" if "maintainer"` shorthand rule above expands
to:

```polar
has_permission(actor: Actor, "push", repository: Repository) if
  has_role(actor, "maintainer", repository);
```

{{% callout "Note" "blue" %}}
  Instances of our application's `User` type will match the `Actor`
  [specializer](polar-syntax#specialization) because of our `actor User {}`
  resource block declaration.
{{% /callout %}}

## Grant roles to other roles

All of the shorthand rules we've written so far have been in the `<permission>
if <role>` form, but we can also write `<role1> if <role2>` rules. This type
of rule is great for situations where you want to express that `<role2>` should
be granted every permission you've granted to `<role1>`.

In the previous snippet, the permissions granted to the `"maintainer"` role are
a superset of those granted to the `"contributor"` role. If we replace the
existing `"read" if "maintainer"` rule with `"contributor" if "maintainer"`,
the `"maintainer"` role still grants the `"read"` permission:

{{< code file="main.polar" hl_lines="10-11" >}}
resource Repository {
  permissions = ["read", "push"];
  roles = ["contributor", "maintainer"];

  # An actor has the "read" permission if they have the "contributor" role.
  "read" if "contributor";
  # An actor has the "push" permission if they have the "maintainer" role.
  "push" if "maintainer";

  # An actor has the "contributor" role if they have the "maintainer" role.
  "contributor" if "maintainer";
}
{{< /code >}}

In addition, any permissions we grant the `"contributor"` role in the future
will automatically propagate to the `"maintainer"` role.

<!-- TODO(gj): better heading -->
## Access role assignments stored in the application

An Oso policy contains authorization *logic*, but the application remains in
control of all authorization *data*. For example, the logic that the
`"maintainer"` role on a repository grants the `"push"` permission lives in the
policy, but Oso doesn't manage the data of which users have been assigned the
`"maintainer"` role for `Repository("Acme App")`. That data stays in the application,
and Oso asks the application for it as needed.

The main question Oso asks is: does `User("Ariana")` have the `"maintainer"` role
on `Repository("Acme App")`? For Oso to be able to ask this question, we need to
implement a `has_role()` rule in the policy:

{{< literalInclude
  dynPath="policy_path"
  from="docs: begin-has_role"
  to="docs: end-has_role"
>}}

`role in user.{{% exampleGet "roles" %}}` iterates over a user's assigned roles
and `role.{{% exampleGet "role_name" %}} = name and role.{{% exampleGet
"role_resource" %}} = resource` succeeds if the user has been assigned the
`name` role for `resource`.

{{< callout "Note" "blue" >}}
  <!-- TODO(gj): spacing seems off in these callouts -->
  <div class="pb-4"></div>
  The body of this rule will vary according to the way roles are stored in your
  application. The data model for our GitClub example is as follows:

  <div class="pb-4"></div>

  {{< literalInclude
    dynPath="app_path"
    from="docs: begin-types"
    to="docs: end-types"
  >}}

  If, for example, repository roles and organization roles were stored
  separately instead of in a heterogeneous set, we might define a pair of
  `has_role()` rules, one for each role type:

  <!-- TODO(gj): why do I need to dedent this? -->
  {{< code codeLang="polar" >}}
has_role(user: User, name: String, repository: Repository) if
  role in user.{{% exampleGet "repository_roles" %}} and
  role.{{% exampleGet "role_name" %}} = name and
  role.{{% exampleGet "role_repository" %}} = repository;

has_role(user: User, name: String, organization: Organization) if
  role in user.{{% exampleGet "organization_roles" %}} and
  role.{{% exampleGet "role_name" %}} = name and
  role.{{% exampleGet "role_organization" %}} = organization;
  {{< /code >}}
{{% /callout %}}

Our `has_role()` rule can check role assignments on repositories and
organizations, but so far we've only talked about repository roles. Let's
change that and see how Oso can leverage parent-child relationships like the
one between `Repository` and `Organization` to grant a role on a child resource
to a role on the parent.

## Inherit a role on a child resource from the parent

If you've used ~~GitHub~~ *GitClub* before, you know that having a role on an
organization grants certain roles and permissions on that organization's
repositories. For example, a user is granted the `"maintainer"` role on a
repository if they're assigned the `"owner"` role on the repository's parent
organization. This is how you write that rule with Oso:

{{< literalInclude
  dynPath="policy_path"
  lines="22-26,37-45"
  hl_lines="4,8-9,12-13"
  ellipsis="  # ..."
>}}

First, we declare that every `Repository` has a `"parent"` relation that
references an `Organization`:

{{< literalInclude
  dynPath="policy_path"
  from="docs: begin-relations"
  to="docs: end-relations"
>}}

This is a dictionary where each key is the name of the relation and each value
is the relation's type.

Next, we write a `has_relation()` rule that tells Oso how to check if an
organization has the `"parent"` relation with a repository:

{{< literalInclude
  dynPath="policy_path"
  from="docs: begin-has_relation"
  to="docs: end-has_relation"
>}}

<!-- TODO(gj): better phrasing for the next sentence -->
In this case, an organization is the `"parent"` of a repository if the
repository's `organization` field points to it.

{{% callout "Note" "blue" %}}
  Note that the resource where we declared the relationship, `Repository`, is
  the *third* parameter and the related resource, `Organization`, is the
  *first*.

  This ordering was chosen to mirror the ordering of the expanded forms for
  `has_role()` and `has_permission()`, where the resource for which the actor
  has the role or permission is the third argument:

  ```polar
  has_role(actor: Actor,                   name: String, resource: Resource) if ...

  has_permission(actor: Actor,             name: String, resource: Resource) if ...

  has_relation(related_resource: Resource, name: String, resource: Resource) if ...
  ```

{{% /callout %}}

Finally, we add a shorthand rule that involves the `"maintainer"` repository
role, the `"owner"` organization role, and the `"parent"` relation between the
two resource types:

{{< literalInclude
  dynPath="policy_path"
  lines="22-26,37-39"
  hl_lines="4,8-9"
  ellipsis="  # ..."
>}}

## Add an `allow()` rule

At this point, the policy is almost fully functional. All that's left is adding
an `allow()` rule:

{{< literalInclude
  dynPath="policy_path"
  from="docs: begin-allow"
  to="docs: end-allow"
>}}

This is a typical `allow()` rule for a policy using resource blocks: an actor is allowed to
perform an action on a resource if the actor *has permission* to perform the
action on the resource. <!-- And an actor has permission to perform an action
on a resource if the actor is assigned a role that grants that permission. -->

This `allow()` rule serves as the entrypoint when we query our policy via Oso's
enforcement methods like {{% apiDeepLink class="Oso" %}}{{% exampleGet "authorize_method_name" %}}{{%
/apiDeepLink %}}:

{{% exampleGet "authorize" %}}

## Baby Got RBAC

Our complete policy looks like this:

{{< literalInclude dynPath="policy_path" >}}

If you'd like to play around with a more fully-featured version of this policy
and application, check out the GitClub repository on [GitHub][GitClub].

If you're building with microservices, read about [how to model roles using Oso Cloud](https://www.osohq.com/docs/guides/modeling-building-blocks).
