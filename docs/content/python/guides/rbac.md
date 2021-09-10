---
title: Build Role-Based Access Control (RBAC)
weight: -1
description: |
  Build role-based access control (RBAC) with Oso's built-in authorization
  modeling features.
aliases:
  - todo
---

# Build Role-Based Access Control (RBAC)

Role-based access control (RBAC) is so ubiquitous that Oso provides special
syntax for modeling RBAC. This syntax makes it easy to create a role-based
authorization policy with roles and permissions -- for example, declaring that
the `"maintainer"` role on a repository allows a user to `"push"` to that
repository.

In this guide, we'll walk through building an RBAC policy for [GitClub][].

[GitClub]: https://github.com/osohq/gitclub

## Declare application types as actors and resources

Oso makes authorization decisions by determining if an **actor** can perform an
**action** on a **resource**:

- **Actor**: who is performing the action? `User(id=1)`
- **Action**: what are they trying to do? `"push"`
- **Resource**: what are they doing it to? `Repository(id=2)`

The first step of building an RBAC policy is telling Oso which application
classes are **actors** and which are **resources**. Our example app has a pair
of **resource** types that we want to control access to, `Organization` and
`Repository`. We declare both as resources as follows:

```polar
resource Organization {}

resource Repository {}
```

Our app also has a `User` class that will be our lone **actor** type:

```polar
actor User {}
```

This piece of syntax is called a *resource block*, and it performs two
functions: it identifies the type as an **actor** or a **resource**, and it
provides a centralized place to declare roles and permissions for that
particular type.

{{% callout "Note" "blue" %}}
  For every resource block, we also need to register the class with Oso:

  ```py {hl_lines=[3,4,5]}
  oso = Oso()

  oso.register_class(Organization)
  oso.register_class(Repository)
  oso.register_class(User)

  oso.load_files(["rbac.polar"])
  ```

{{% /callout %}}

## Declare roles and permissions

In GitClub, users can perform actions such as `"delete"`-ing an organization or
`"push"`-ing to a repository. Users can also be assigned roles for either type
of resource, such as the `"owner"` role for an `Organization` or the
`"maintainer"` role for a `Repository`.

Inside the curly braces of each `resource` block, we declare the roles and
permissions for that resource:

```polar
resource Organization {
  roles = [ "owner" ];
}

resource Repository {
  permissions = [ "read", "push" ];
  roles = [ "contributor", "maintainer" ];
}
```

<!-- TODO(gj): transition -->

## Grant permissions to roles

Next, we're going to write *shorthand rules* that grant permissions to roles.
For example, if we grant the `"push"` permission to the `"maintainer"` role in
the `Repository` resource block, then a user who's been assigned the
`"maintainer"` role for a particular repository can `"push"` to that
repository. Here's our `Repository` resource block with a few shorthand rules
added:

```polar
resource Repository {
  permissions = [ "read", "push" ];
  roles = [ "contributor", "maintainer" ];

  "push" if "maintainer";
  "read" if "maintainer";
  "read" if "contributor";
}
```

Shorthand rules expand to regular [Polar rules](polar-syntax#rules) when a
policy is loaded. The `"push" if "maintainer"` shorthand rule above expands
to:

```polar
has_permission(actor: Actor, "push", repository: Repository) if
  has_role(actor, "maintainer", repository);
```

{{% callout "Note" "blue" %}}
  Instances of our application's `User` class will match the `Actor`
  [specializer](polar-syntax#specialization) because of our `actor User {}`
  resource block declaration.
{{% /callout %}}

<!-- TODO(gj): transition -->

## Grant roles to other roles

All of the shorthand rules we've written so far have been in the `<permission>
if <role>` form, but we can also write `<role1> if <role2>` rules. This type
of rule is great for situations where you want to express that `<role2>` should
be granted every permission you've granted to `<role1>`.

In the previous snippet, the permissions granted to the `"maintainer"` role are
a superset of those granted to the `"contributor"` role. If we replace the
existing `"read" if "maintainer"` rule with `"contributor" if "maintainer"`,
the `"maintainer"` role still grants the `"read"` permission:

```polar
resource Repository {
  permissions = [ "read", "push" ];
  roles = [ "contributor", "maintainer" ];

  "push" if "maintainer";
  "read" if "contributor";

  "contributor" if "maintainer";
}
```

In addition, any permissions we grant the `"contributor"` role in the future
will automatically propagate to the `"maintainer"` role.

<!-- TODO(gj): better heading -->
## Access role assignments stored in the application

An Oso policy contains authorization *logic*, but the application remains in
control of all authorization *data*. For example, the logic that the
`"maintainer"` role on a repository grants the `"push"` permission lives in the
policy, but Oso doesn't manage the data of which users have been assigned the
`"maintainer"` role for `Repository(id=2)`. That data stays in the application,
and Oso asks the application for it as needed.

The main question Oso asks is: does `User(id=1)` have the `"maintainer"` role
on `Repository(id=2)`? For Oso to be able to ask this question, we need to
implement a `has_role()` rule in the policy:

```polar
has_role(user: User, name: String, resource: Resource) if
  role in user.roles and
  role matches { name: name, resource: resource };
```

`role in user.roles` iterates over a user's assigned roles and `role matches {
name: name, resource: resource }` succeeds if the user has been assigned the
`name` role for `resource`.

{{% callout "Note" "blue" %}}
  The body of this rule will vary according to the way roles are stored in your
  application. The data model for our GitClub example is as follows:

  ```py
  @dataclass(frozen=True)
  class Organization:
      name: str

  @dataclass(frozen=True)
  class Repository:
      name: str
      organization: Organization

  @dataclass(frozen=True)
  class Role:
      name: str
      resource: Union[Repository, Organization]

  @dataclass
  class User:
      name: str
      roles: Set[Role]

      def assign_role_for_resource(self, name, resource):
          self.roles.add(Role(name, resource))
  ```

  If, for example, repository roles and organization roles were stored
  separately instead of in a heterogeneous set, we might define a pair of
  `has_role()` rules, one for each role type:

  ```polar
  has_role(user: User, name: String, repository: Repository) if
    role in user.repository_roles and
    role matches { name: name, resource: resource };

  has_role(user: User, name: String, organization: Organization) if
    role in user.organization_roles and
    role matches { name: name, resource: resource };
  ```

{{% /callout %}}

Our `has_role()` rule can check role assignments on repositories and
organizations, but so far we've only talked about repository roles. Let's
change that and see how Oso can leverage parent-child relationships like the
one between `Repository` and `Organization` to grant a role on a child resource
to a role on the parent.

<!-- TODO(gj): better heading -->
## Grant a role on a child resource to a role on the child's parent

If you've used ~GitHub~ *GitClub* before, you know that having a role on an
organization grants certain roles and permissions on that organization's
repositories. For example, a user is granted the `"maintainer"` role on a
repository if they're assigned the `"owner"` role on the repository's parent
organization. This is how you write that rule with Oso:

{{< code file="rbac.polar" >}}
resource Repository {
  # ...

  relations = { parent: Organization };

  "maintainer" if "owner" on "parent";
}

has_relation(organization: Organization, "parent", repository: Repository) if
  organization = repository.organization;
{{< /code >}}

First, we declare that the `"parent"` relation for a `Repository` is an
`Organization`:

```polar
relations = { parent: Organization };
```

This is a dictionary where each key is the name of the relation and each value
is the relation's type.

Next, we write a `has_relation()` rule that tells Oso how to check if an
organization has the `"parent"` relation with a repository:

```polar
has_relation(organization: Organization, "parent", repository: Repository) if
  organization = repository.organization;
```

<!-- TODO(gj): better phrasing for the next sentence -->
In this case, an organization is the `"parent"` of a repository if the
repository's `organization` field points to it.

Finally, we add a shorthand rule that involves the `"maintainer"` repository
role, the `"owner"` organization role, and the `"parent"` relation between the
two resource types:

```polar
"maintainer" if "owner" on "parent";
```

## Add an `allow()` rule

At this point, the policy is almost fully functional. All that's left is adding
an `allow()` rule:

```polar
allow(actor, action, resource) if
  has_permission(actor, action, resource);
```

This is a typical `allow()` rule for an RBAC policy: an actor is allowed to
perform an action on a resource if the actor *has permission* to perform the
action on the resource. <!-- And an actor has permission to perform an action
on a resource if the actor is assigned a role that grants that permission. -->

This `allow()` rule serves as the entrypoint when we query our policy via Oso's
enforcement methods like {{% apiDeepLink class="Oso" %}}authorize{{%
/apiDeepLink %}}:

```py
oso.authorize(User(id=1), "push", Repository(id=2))
```

## Baby Got RBAC

Our complete policy looks like this:

{{< code file="rbac.polar" >}}
allow(actor, action, resource) if
  has_permission(actor, action, resource);

has_role(user: User, name: String, resource: Resource) if
  role in user.roles and
  role matches { name: name, resource: resource };

actor User {}

resource Organization {
  roles = [ "owner" ];
}

resource Repository {
  permissions = [ "read", "push" ];
  roles = [ "contributor", "maintainer" ];
  relations = { parent: Organization };

  "read" if "contributor";
  "push" if "maintainer";

  "contributor" if "maintainer";

  "maintainer" if "owner" on "parent";
}

has_relation(organization: Organization, "parent", repository: Repository) if
  organization = repository.organization;
{{< /code >}}

If you'd like to play around with a more fully-featured version of this policy
and application, check out the GitClub repository on [GitHub][GitClub].
