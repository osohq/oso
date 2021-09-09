---
title: Model your authorization policy
description: |
    Authorization in Oso starts with the policy. Authorization policies
    define the resources that you want to control access to, and rules that
    specify when an actor (the user making the request) can perform an
    action on a resource. Because Oso is a library, you can write
    authorization policies directly over the same data types that are
    already used in your application.
weight: 1
---

# Model your authorization policy

Authorization in Oso starts with the policy. Authorization *policies*
define the *resources* that you want to control access to, and rules
that specify when an *actor* (the user making the request) can perform
an *action* on a *resource*. Because Oso is a library, you can write
authorization policies directly over the same data types that are
already used in your application.

In this guide, we'll cover how to express your authorization logic using
Oso. We'll use GitClub (our example application) as an example, but you
can follow along with your application.

{{% 1on1 %}}

For help deciding what type of authorization model to use, chat with our
team.

{{% /1on1 %}}

## Installing Oso

The Oso library evaluates authorization policies.

The Python version of Oso is available on [PyPI](https://pypi.org/project/oso/)
and can be installed using `pip`:

```console
$ pip install oso=={{< version >}}
```

For more detailed installation instructions, see
[installation](/reference/installation).

## Write a policy

Policies are files that are packaged with the rest of your application
code. The Oso library loads and evaluates policy files when your
application runs. Now that you've installed Oso, create a policy:

```python
from oso import Oso

from gitclub.models import User, Repository

# Initialize the Oso object. This object is usually used globally throughout
# an application, and it is thread-safe.
oso = Oso()

# Tell Oso about the data that you will authorize. These types can be referenced
# in the policy.
oso.register_class(User)
oso.register_class(Repository)

# Load your policy file.
oso.load_files(["policies/main.polar"])
```

To setup Oso in your app, you must...

- initialize `Oso`: Usually Oso will be initialized globally and used
during every request to enforce authorization.
- tell Oso what data types you will authorize: With Oso, you express
  authorization logic over the data types used in your application. Often
  these will be model classes, but they could be any type of data or
  resource you want to control access to.
- load your policy: Oso policies are written in policy files. Call
  `load_files` with the paths to your policy files.

## Define your resources, permissions and roles

Authorization controls access to *resources* in your application. In
this guide, we're going to show how to implement the most common
authorization model, role-based access control, with Oso. See [How to:
modeling](TODO) **TODO** to see how to implement other authorization
models in Oso.

To start with, you need to define resource blocks in your policy. Let's
say we have an application like GitHub that includes a `Repository`
resource. Define the `Repository` resource in `policy.polar`:

TODO actor definition

```polar
resource Repository {
	permissions = ["read", "pull"];
	roles = ["member"];

	# A user has the "read" permission if they have
	# the "member" role.
	"read" if "member";

	# A user has the "pull" permission if they have the
	# "member" role.
	"pull" if "member";
}
```

This policy declares `"read"` and `"pull"` as permissions for the
`Repository` resource and gives users the `"read"` and `"pull"`
permissions if the user has the `"member"` role.

The `"read" if "member";` statement is an example of a *shorthand rule.*
Add a `"maintainer"` role and give it its own permissions by adding some
more shorthand rules:

```polar
resource Repository {
	permissions = ["read", "pull", "push"];
	roles = ["member", "maintainer"];

	"read" if "member";
	"pull" if "member";

	# A user has the "push" permission if they have the
	# "maintainer" role.
	"push" if "maintainer";

	# A user has the member role if they have
	# the maintainer role.
	"member" if "maintainer";
}
```

The last rule we added is between two roles: A user has the "member"
role and all permissions associated with it if they have the
"maintainer" role.

### Giving your users roles

Now that we've finished our policy, we must associate users with roles
in our application. Oso doesn't manage authorization data. The data
stays in your application's data store.

All the data we've defined so far in the policy is static: it isn't
changed by end users of the application. Only the development team
modifies permission associations with roles, and the list of roles for
each resource. But, some parts of this policy must be dynamic: the
association of users with a role.

Write a `has_role` rule to tell Oso whether your users have a particular
role.

```polar
has_role(user: User, role_name, repo: Repo) if
  role in user.repo_roles and
  role_name = role.name and
  role.repo_id = repo.id;
```

{{% minicallout %}}
This is an example of a full Polar rule. We'll go more into writing
rules in the **Write your own policy** guide. __TODO LINK__
{{% /minicallout %}}

The `has_role` rule uses the user object passed into Polar by your
application to lookup roles. In this example, Polar will access the
`repo_roles` field on the `user` object and look up the role names that
the user has.

```python
<python implementation of data model>
```

### Allowing access

Oso policies have a special rule: the `allow` rule. The `allow` rule is
the entrypoint to the policy, and is used by the Oso library to enforce
authorization.

The resource blocks you define grant users permissions using
`has_permission` rule. To use permissions for authorization, you must
define an `allow` rule:

```polar
allow(actor, action, resource) if
	has_permission(actor, action, resource);
```

### The complete policy

```polar
resource Repository {
	permissions = ["read", "pull", "push"];
	roles = ["member", "maintainer"];

	"read" if "member";
	"pull" if "member";

	# A user has the "push" permission if they have the
	# "maintainer" role.
	"push" if "maintainer";

	# A user is given the member role if they have
	# the maintainer role.
	"member" if "maintainer";
}

has_role(user: User, role_name, repo: Repository) if
	role in user.repo_roles and
  role_name = role.name and
  role.repo_id = repo.id;

allow(user: User, action, resource) if
	has_permission(actor, action, resource);
```

### What's next

Now, we've setup our policy. Let's see how to enforce authorization
decisions using it.
