---
title: Model Your Authorization Logic
description: |
    Authorization in Oso starts with the policy. Authorization policies
    define the resources that you want to control access to, and rules that
    specify when an actor (the user making the request) can perform an
    action on a resource. Because Oso is a library, you can write
    authorization policies directly over the same data types that are
    already used in your application.
weight: 1
---

# Model Your Authorization Logic

Authorization in Oso starts with the policy. To model authorization with
Oso, you write a policy in Polar - Oso's declarative policy language.
The policy defines the *resources* that you want to control access to
and includes rules governing access to them.

In this guide, we'll cover how to express your authorization logic using
Oso. We'll use GitClub (our example application) as an example, but you
can follow along with your application.

## Install Oso

The Oso library evaluates authorization policies.

The {{% lang %}} version of Oso is available on {{% exampleGet "installLink" %}}
and can be installed using {{% exampleGet "installer" %}}:

{{% exampleGet "installInstruction" %}}

For more detailed installation instructions, see
[installation](/reference/installation).

## Create a Policy

Policies are files that are packaged with the rest of your application
code. Oso loads and evaluates policy files when your
application runs. Now that you've installed Oso, create a policy file
called `main.polar` and use it in your app:

{{< literalInclude dynPath="pathOso" fallback="todo" >}}

## Define Resources, Permissions, and Roles

Authorization controls access to *resources* in your application. In
this guide, we're going to show how to implement the most common
authorization model, role-based access control, with Oso. See the
[guides](/guides) section for how to implement other authorization models with
Oso.

To start with, you need to define resource blocks in your policy. Let's
say we have an application like GitHub that includes a `Repository`
resource. Define the `Repository` resource in `main.polar`:

```polar
actor User {}

resource Repository {
    permissions = ["read", "push"];
    roles = ["contributor", "maintainer"];

    # A user has the "read" permission if they have the
    # "contributor" role.
    "read" if "contributor";

    # A user has the "push" permission if they have the
    # "maintainer" role.
    "push" if "maintainer";
}
```

This policy declares `"read"` and `"push"` as permissions for the
`Repository` resource and assigns them to roles. We also tell Oso
that our `User` class is an `actor` with the `actor User {}`
declaration.

The `"read" if "contributor";` statement is an example of a *shorthand rule.*
Add an `"admin"` role and give it its own permissions by adding some
more shorthand rules:

```polar
resource Repository {
    permissions = ["read", "push", "delete"];
    roles = ["contributor", "maintainer", "admin"];

    "read" if "contributor";
    "push" if "maintainer";

	# A user has the "delete" permission if they have the
	# "admin" role.
	"delete" if "admin";

	# A user has the "maintainer" role if they have
	# the "admin" role.
    "maintainer" if "admin";

	# A user has the "contributor" role if they have
	# the "maintainer" role.
    "contributor" if "maintainer";
}
```

The last rules we added are between two roles: A user has the
`"maintainer"` role and all permissions associated with it if they have the
`"admin"` role, and the same for `"contributor"` and `"maintainer"`.

### Give Your Users Roles

Now that we've written the core of our policy, we must associate users with roles
in our application. Oso doesn't manage authorization data. The data
stays in your application's existing data store.

{{% minicallout %}}
**Static and dynamic data**: All the data we've defined so far in the
policy is static: it isn't changed by end users of the application. The
development team modifies permission associations with roles and the
list of roles for each resource by updating the policy. But, some parts
of this policy must be dynamic: the association of users with a role.
{{% /minicallout %}}

Write a `has_role` rule to tell Oso whether your users have a particular
role:

```polar
has_role(user: User, role_name: String, repo: Repository) if
  role in actor.roles and
  role_name = role.name and
  repository = role.repository;
```

{{% minicallout %}}
This is an example of a full Polar rule. We'll go more into writing
rules in the [Write Polar rules](write-rules).
{{% /minicallout %}}

The `has_role` rule uses the user object passed into Oso by your
application to lookup roles. In this example, Polar will access the
`roles` field on the `user` object and look up the role names that
the user has. Here's an example {{% lang %}} data model
that could be used by this rule. In your application, you'll
likely use your existing User model to maintain this information.

{{< literalInclude
    dynPath="pathModels"
    fallback="todo"
    from="docs: start"
    to="docs: end" >}}

### Allow Access

Oso policies have a special rule: the `allow` rule. The `allow` rule is
the entrypoint to the policy, and is used by the Oso library to check if an
*actor* (the user making a request) can perform an *action* on a *resource*.

The resource blocks you wrote define your authorization model. For
example, `"read" if "contributor"` says a user has the `"read`"
permission if they have the `"contributor"` role.

You can check for this condition by calling the `has_permission` rule:
`has_permission(user, "read", repository)`.

To connect this with the `allow` entrypoint, you must write the
following:

```polar
# Allow access if users have the required permission,
# as defined by resource blocks.
allow(actor, action, resource) if
	has_permission(actor, action, resource);
```

### The Complete Policy

```polar
actor User {}

resource Repository {
    permissions = ["read", "push", "delete"];
    roles = ["contributor", "maintainer", "admin"];

    "read" if "contributor";
    "push" if "maintainer";
    "delete" if "admin";

    "maintainer" if "admin";
    "contributor" if "maintainer";
}

has_role(actor: User, role_name: String, repository: Repository) if
    role in actor.roles and
    role_name = role.name and
    repository = role.repository;

allow(actor, action, resource) if
    has_permission(actor, action, resource);
```

### What's Next

{{% 1on1 %}}

For help deciding what type of authorization model to use, chat with our
team.

{{% /1on1 %}}

Now that we've setup our policy, let's see how we can enforce it!
