---
title: Getting started
weight: 2
description: >
    Get started with Oso Roles for Node.js
---

# Getting started

When managing access to resources within an application, it can be useful to
group permissions into **roles** and assign these roles to users. This is
known as **Role-Based Access Control (RBAC).** The Oso library
comes with built-in configuration for role-based access control.

In this guide, we'll walk through the basics of starting to use the
roles feature.

## Setting up the Oso instance

First, we'll cover some of the basics of integrating Oso into your
application.

The `Oso` class is the entrypoint to using Oso in our application.
We usually will have a global instance that is created
during application initialization and shared across requests.

### Loading our policy

Oso uses the [Polar language](/reference/polar/polar-syntax) to define authorization
policies. An authorization policy specifies what requests are allowed and what data a
user can access. The policy is stored in a Polar file, along with your code.

Load the policy with the `Oso.loadFile()` function.

```js {hl_lines=[3]}
const { Oso } = require('oso');
const oso = new Oso();
await oso.loadFile("authorization.polar");
```

### Enable Oso Roles

In order to enable the built-in roles features, we call the
`Oso.enableRoles()` method:

```py
await oso.enableRoles()
```

{{% callout "Load policies before enabling roles" "blue" %}}

Oso will validate your roles configuration when you call `enableRoles()`.
You must load all policy files before enabling roles.

{{% /callout %}}

## Controlling access with roles

Now, let's write our first rules that use role-based access control. To
set up the role library, we must:

1. Add role and resource configurations to our policy.
2. Use the `role_allows` method in our policy.
3. Assign roles to users.

### Configuring our first resource

Roles in Oso are scoped to resources. A role is a grouping of
permissions -- the actions that may be performed on that resource.
Roles are assigned to actors to grant them all the permissions the role has.

We define resources in Polar using the `resource` rule. The `Org`
resource represents an Organization in the GitClub example application.
Let's walk through the resource definition for `Org`.

```polar
resource(_type: Org, "org", actions, roles) if
```

The rule head has 4 parameters:

- `_type` is the JavaScript class the resource definition is associated with.
- `"org"` is the identifier for this resource type (this can be any string
  you choose).
- `actions` is a list enumerating all the
  actions that may be performed on the resource.
- `roles` is a dictionary defining all the
  roles for this resource.

In our rule body, we first define the list of available actions for this
resource:

```polar
resource(_type: Org, "org", actions, roles) if
    actions = ["read", "create_repo"] and
    roles = {
        ...
    };
```

Now, we define our roles. Roles are defined in a dictionary that maps
the role name to a role configuration.

```polar
resource(_type: Org, "org", actions, roles) if
    actions = ["read", "create_repo"] and
    roles = {
        member: {
            permissions: ["read"],
        },
        owner: {
            permissions: ["read", "create_repo"],
        }
    };
```

This resource definition defines two roles:

- _member_: Has the `read` permission.
- _owner_: Has the `read` and `create_repo` permissions.

Permissions are actions associated with a resource type. A permission can
directly reference an action defined in the same resource. Later, we'll
see how to leverage relationships between resources to grant a role a
permission defined on a different resource

{{% callout "resource(...) is just a rule" "blue" %}}

The `resource` definition is just a regular Polar rule. That's why it
has an `if` and `and` between variable assignments. `actions` and
`roles` are unbound parameters, meaning they can be assigned inside of
the rule body.

We could have written this rule without a body:

```polar
resource( _type: Org, "org", ["read", "create_repo"],
    {
        member: {
            permissions: ["read"],
        },
        owner: {
            permissions: ["read", "create_repo"],
        }
    }
);
```

But we think the expanded form is clearer.

{{% /callout %}}

### Adding `role_allows` to our policy

To allow access based on roles, we add the following `allow` rule

```polar
allow(actor, action, resource) if
    role_allows(actor, action, resource);
```

Oso will now allow access to any resource that is allowed based on the
role definitions.

### Assigning roles to users

Now we've configured roles and set
up our policy. For users to have
access, we must assign them roles.

You can use your own data models for roles with Oso. You just need to tell us
what roles a user has for a particular resource
through the `actor_has_role_for_resource` rule. As an example, we might
add a method onto the user that returns a list of roles for that user:

```py
const ROLES = {
    "alice": [
        {"name": "member", "resource": Org.orgs[0]},
        {"name": "owner", "resource": Org.orgs[1]},
    ],
    "bob": [{"name": "owner", "resource": Org.orgs[2]}],
}

class User {
    constructor(name) {
        this.name = name;
    }

    getRoles() {
        return ROLES[this.name];
    }
}
```

And the `actor_has_role_for_resource` would be implemented as:

```polar
actor_has_role_for_resource(actor, role_name, resource) if
    role in actor.getRoles() and
    role_name = role.name and
    resource = role.resource;
```

The `actor_has_role_for_resource` is evaluated with `actor` bound to the same actor
that you call the `allow` rule with, typically an instance of some `User` model.

`role_name` and `resource` are "output parameters".
In the body of the `actor_has_role_for_resource` rule, you
should unify `role_name` with the name of the actor's role and
`resource` with the instance the actor has the role for. In
the example above, Bob has the `"owner"` role for the
`Org.orgs[2]` resource, so when `role in actor.get_roles()` is
evaluated with Bob as the `actor`, `role.name` will return `"owner"`
and `role.resource` will return `Org.orgs[2]`.

### Implying roles

The `"owner"` role is a more permissive role than `"member"`. It
covers all the permissions of `"member"`, with some additional
permissions granted (`"create_repo"`) in our example.

Instead of duplicating the permissions, we can represent this
relationship in our policy using **implied roles**.

```polar
resource(_type: Org, "org", actions, roles) if
    actions = ["read", "create_repo"] and
    roles = {
        member: {
            permissions: ["read"],
        },
        owner: {
            permissions: ["create_repo"],
            implies: ["member"]
        }
    };
```

The `"owner"` role now implies the `"member"` role. Any user with the
`"owner"` role will be granted all permissions associated with both
roles.

Here's the full `Org` resource definition from the GitClub example app:

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/authorization.polar"
    from="docs: begin-org-resource"
    to="docs: end-org-resource"
    hlFrom="docs: begin-org-resource-highlight"
    hlTo="docs: end-org-resource-highlight"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
>}}

Notice the `"repo:reader"` and `"repo:admin"` implications. These are
roles defined on another resource, `Repo`. In the next guide, we'll
see how to set up **cross resource implied roles** like these!

## Have feedback?

If at any point you get stuck, drop into our
[Slack](https://join-slack.osohq.com/) or <a href="mailto:engineering@osohq.com">send an email</a> to our engineering
team and we'll unblock you.
