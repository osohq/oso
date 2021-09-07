---
title: Getting started
weight: 2
description: >
    Get started with Oso Roles for SQLAlchemy
---

# Getting started

When managing access to resources within an application, it can be useful to
group permissions into **roles**, and assign these roles to users. This is known
as **Role-Based Access Control (RBAC).** The Oso Roles for SQLAlchemy feature
extends [Oso Roles for Python](/guides/roles) by exposing an API for managing
role assignments and reading roles data.

In this guide, we'll walk through the basics of starting to use the
Oso Roles for SQLAlchemy feature, using the
[GitClub](https://github.com/osohq/gitclub)
application as an example. GitClub is a SPA (single-page application)
that uses Flask and SQLAlchemy for the backend, with a React frontend.
To install **GitClub** to run alongside this tutorial, follow the
[`README`](https://github.com/osohq/gitclub/tree/main/backends/flask-sqlalchemy#readme).

## Setting up the Oso instance

{{% callout "Already using `sqlalchemy-oso`?" "blue" %}}

We're going to cover some of the basics of using Oso and the
`sqlalchemy-oso` library. If you're already familiar with this [skip
ahead to configuring
roles](#controlling-access-with-roles).

{{% /callout %}}

<!-- @TODO(gj): (nit) dissonance between 'our' & 'your' throughout this doc. -->

Oso is a library that we use in our application for authorization. It
requires no additional infrastructure. Instead, the SQLAlchemy library
helps you authorize data in your existing data store. Data required for
authorization (like role assignment) is stored in the same database as
the rest of your application data.

First, we'll cover some of the basics of integrating Oso into your
application.

The `sqlalchemy_oso.SQLAlchemyOso` class is the entrypoint to using Oso in our
SQLAlchemy application. We usually will have a global instance that is created
during application initialization and shared across requests. In a Flask
application, you may attach this instance to the global flask object, or the
current application if it needs to be accessed outside of the application
initialization process.

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/__init__.py"
    from="docs: begin-init-oso"
    to="# Enable roles features."
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
>}}

### Registering authorization data types

Oso policies are written over the data in our application that we'd like to
authorize. The policy can directly reference the same types we use in our
application.

Since we are using Oso with SQLAlchemy, we need to make Oso aware of our
SQLAlchemy models. Typically, we would make Oso aware of our application
classes by registering them via `Oso.register_class`, but the `sqlalchemy-oso`
library provides a handy shortcut.

By providing our SQLAlchemy base model to the `sqlalchemy_oso.SQLAlchemyOso`
constructor, all of our SQLAlchemy models that inherit from `Base` will be
automatically registered with Oso.

### Enabling built-in roles

In order to enable the built-in roles feature, we need to pass our app's user
class as well as a SQLAlchemy `sessionmaker` to the
`sqlalchemy_oso.SQLAlchemyOso.enable_roles` method:

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/__init__.py"
    from="docs: begin-init-oso"
    to="# Load authorization policy."
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
>}}

### Loading our policy

Oso uses the Polar language to define authorization policies. An
authorization policy specifies what requests are allowed and what data a
user can access. The policy is stored in a Polar file, along with your
code.

Load the policy with the `Oso.load_file` function.

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/__init__.py"
    from="docs: begin-init-oso"
    to="# Attach SQLAlchemyOso instance to Flask application."
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
>}}


## Writing an authorization rule over a SQLAlchemy model

Now that we've setup Oso, we can write rules in our Polar policy to
control access to SQLAlchemy models. The entrypoint
to a policy is the `allow` rule, which specifies when an `actor` can
perform an `action` on a particular `resource`.

We can write simple rules in our policy, like this one which allows all
users to create new organizations:

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/authorization.polar"
    from="docs: begin-org-create-rule"
    to="docs: end-org-create-rule"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
>}}

This is an `allow` rule. The arguments are the `actor`, `action` and
`resource` being authorized. Our rule has the following behavior for
each argument:

- `_: User`: the `actor` argument must have the `User` type. Since the
  argument is unused we name it `_`.
- `"create"`: the `action` must match the string literal `"create"`
- `_: Org`: the `resource` argument must have the `Org` type.

Notice that `User` and `Org` are SQLAlchemy models. We can reference
these in our policy because we registered all models with `SQLAlchemyOso(Base)`!

For more on policy basics see our [writing policies
guide](/getting-started/policies).

## Enforcing authorization in our routes

To perform authorization, we use the `Oso.is_allowed` method. Here's an
example in our Org creation handler:

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/routes/orgs.py"
    from="docs: begin-is-allowed"
    to="docs: end-is-allowed"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
>}}

The policy is consulted to see if there is an `allow` rule that permits
access for the `actor`, `action` and `resource` arguments passed to
`is_allowed`. We can access types and attributes of the objects passed
into our rules from within the policy, just like we can in Python!
More on this [here]({{< ref
"/getting-started/policies.md#instances-and-fields" >}}).

## SQLAlchemy session setup

Oso integrates with SQLAlchemy [sessions](https://docs.sqlalchemy.org/en/13/orm/session_basics.html#what-does-the-session-do) to authorize access to models.

In a typical application, we may have one SQLAlchemy session per
request. Often this is accomplished with a session factory that is
scoped to the [current
thread](https://docs.sqlalchemy.org/en/13/orm/contextual.html).

When using Oso, you should use `sqlalchemy_oso.authorized_sessionmaker`
to create _authorized sessions_. An authorized session applies authorization
to all _read_ queries.

### Using the authorized session

The authorized session is used for fetching data from the database that
must be limited to the current user. When performing authorization, Oso
uses [data filtering](/guides/data_access) to translate the
policy's rules into a SQLAlchemy query. Only authorized objects will be
fetched from the database.

The authorized session is created using `sqlalchemy_oso.authorized_sessionmaker`.
The `authorized_sessionmaker` requires as input: the Oso instance, the current user,
and what permissions to check.

In GitClub, we initialize the session on the application

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/__init__.py"
    from="docs: begin-authorized-session"
    to="docs: end-authorized-session"
    gitHub="https://github.com/osohq/gitclub"
    >}}

provide a helper decorator to define the permissions to check

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/routes/helpers.py"
    from="docs: begin-session-decorator"
    to="docs: end-session-decorator"
    gitHub="https://github.com/osohq/gitclub"
    >}}

and we can then issue regular SQLAlchemy queries to load authorized data in our
route handlers

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/routes/repos.py"
    from="docs: begin-repo-index"
    to="docs: end-repo-index"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
    >}}

### Making a query without authorization

We can make queries without any authorization by setting the `checked_permissions=None`
parameter. We pass this through in the `@session` decorator:

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/routes/orgs.py"
    from="docs: begin-is-allowed"
    to="docs: end-is-allowed"
    gitHub="https://github.com/osohq/gitclub"
>}}

In this case, we are still able to use the regular `oso.is_allowed` to authorize
individual actions.

## Controlling access with roles

Now, let's write our first rules that use role-based access control. To
setup the role library, we must:

1. Persist role configuration to our database.
2. Add role and resource configurations to our policy.
3. Use the `Roles.role_allows` method in our policy.
4. Assign roles to users.

### Persisting roles configuration

Oso stores role and permission configuration in your database alongside
the rest of your SQLAlchemy tables. Additional models are added to your
`Base` class metadata when calling `SQLAlchemyOso.enable_roles`. The schema for these
models can be created with [`MetaData.create_all`](https://docs.sqlalchemy.org/en/13/core/metadata.html#sqlalchemy.schema.MetaData.create_all).

In addition to the schema, we still must persist the role configuration
from the policy into the database. We do this with the
`OsoRoles.synchronize_data` method. The `synchronize_data` method will
replace all role configuration to reflect the `resource` definitions in
the policy.

In our sample app, we call `synchronize_data` in initialization of our app:

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/__init__.py"
    from="docs: begin-configure"
    to="docs: end-configure"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
    >}}

{{% callout "Going to production" "orange" %}}

Since `OsoRoles.synchronize_data()` performs bulk database operations, a
production application should call it as part of the deployment process
in a script.

{{% /callout %}}

{{% callout "Improved role configuration migrations coming soon" "green" %}}

Currently, `OsoRoles.synchronize_data` deletes and replaces all role
configuration in the database. In a future release, we will have a
migration tool that only synchronizes changes to the database, and warns
when removing roles or permissions that are in use.

Roles and permissions are stored in the database to allow creation of
dynamic role and permission assignments in a future release.

{{% /callout %}}

### Configuring our first resource

Roles in Oso are scoped to resources. A role is a grouping of
permissions that may be performed on that resource. Roles are assigned
to actors to grant them all the permissions the role has.

We define resources in Polar using the `resource` rule. The `Org`
resource represents an Organization in the GitClub example application.
Let's walk through the resource definition for `Org`.

```polar
resource(_type: Org, "org", actions, roles) if
```

The rule head has 4 parameters:

- `_type` is the SQLAlchemy model the resource definition is associated with.
- `"org"` is the identifier for this resource type (this can be any string
  you choose).
- `actions` are set in the rule body. The `actions` list defines all the
  actions that may be performed on the resource.
- `roles` are set in the rule body. The `roles` dictionary defines all the
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
see how to add permissions defined on other resources to roles.

{{% callout "resource(...) is just a rule" "blue" %}}

The `resource` definition is just a regular Polar rule. That's why it
has an `if` and `and` between variable assignments. `actions` and
`roles` are unbound parameters, meaning they can be assigned inside of
the rule body.

We could have written this rule without a body, like:

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

But, we think the expanded form is clearer.

{{% /callout %}}

### Adding `role_allows` to our policy

To allow access based on roles, we call `Roles.role_allows` in our
policy:

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/authorization.polar"
    from="docs: begin-role-allow"
    to="docs: end-role-allow"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
>}}

Oso will now allow access to any resource that is allowed based on the
role definitions.

### Assigning roles to users

Now we've configured roles and setup our policy. For users to have
access, we must assign them to roles.

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/routes/role_assignments.py"
    from="docs: begin-role-assignment"
    to="docs: end-role-assignment"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
    hlFrom="docs: begin-role-assignment-highlight"
    hlTo="docs: end-role-assignment-highlight"
    >}}

The `assign_role` method assigns a particular role to a user on a resource.
Often, you'll make this available in a route so that end users can
configure role assignments for their organization. More on this in [end
user configuration](end_user_configuration).

Role assignment is stored your database along with the rest of your
SQLAlchemy managed data.

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

The `"owner"` now implies the `"member"` role. Any user with the
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
roles defined on another resource, Repository. In the next guide, we'll
see how to setup **cross resource implied roles** like these!

## Have feedback?

If at any point you get stuck, drop into our
[Slack](https://join-slack.osohq.com/) or <a href="mailto:engineering@osohq.com">send an email</a> to our engineering
team and we'll unblock you.
