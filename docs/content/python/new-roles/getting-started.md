---
title: Getting started
weight: 2
description: >
    Getting started with SQLAlchemy roles
---

**TODO** i want every API link to go to reference

# Getting started

When managing access to resources within an application, it can be useful to
group permissions into **roles**, and assign these roles to users. This is
known as **Role-Based Access Control (RBAC).** The SQLAlchemy roles
library extends the ``Oso`` core library with built in configuration,
data model and enforcement of roles based access control.

In this guide, we'll walk through the basics of starting to use the
SQLAlchemy roles library, using the
[GitClub](https://github.com/osohq/gitclub-sqlalchemy-flask-react)
application as an example. GitClub is a SPA (single-page application)
that uses Flask and SQLAlchemy for the backend, with a React frontend.

## Installation

To install **GitClub** to run alongside this tutorial, follow the
**TODO**
[README]().

## Setting up the Oso Instance

Oso is a library that we use in our application for authorization.
First, we'll cover some of the basics of integrating Oso into our
application.

The ``Oso`` class is the entrypoint to using Oso in our application. We
usually will have a global instance that is created during application
initialization, and shared across requests. In a Flask application, you
may attach this instance to the global flask object, or the current
application if it needs to be accessed outside of the application
initialization process.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/__init__.py"
    from="docs: begin-init-oso"
    to="Register authorization data"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

### Registering authorization data types

Oso policies are written over the data in our application that we'd like
to authorize. The policy can directly reference the same types we use in
our application. First, we must register our classes with Oso using
`Oso.register_class`.

Since we are using Oso with SQLAlchemy, we need
to make Oso aware of our SQLAlchemy models. Later, we'll write
authorization rules to control access to our models.

The `sqlalchemy_oso.register_models` function helps us register
SQLAlchemy ORM models with Oso.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/__init__.py"
    from="docs: begin-init-oso"
    to="OsoRoles"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

### Loading our policy

Oso uses the Polar language to define authorization policies. An
authorization policy specifies what requests are allowed. The policy is
stored in a Polar file, along with your code.

Load the policy with the `Oso.load_file` function.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/__init__.py"
    from="docs: begin-init-oso"
    to="Attach Oso"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    hlOpts="hl_lines=10-11"
>}}


## Writing an authorization rule over a SQLAlchemy model

Now that we've setup Oso, we can write rules in our Polar policy to
control access to SQLAlchemy models. The entrypoint
to a policy is the `allow` rule, which specifies when a request for a
given `actor`, `action` and `resource` is allowed. For more on
policy basics see our **TODO** [super great policy guide]().

We can write simple rules in our policy, like this one which allows all
users to create new organizations:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: org-create-rule"
    to="end: org-create-rule"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

This is an `allow` rule. The arguments are the `actor`, `action` and
`resource` being authorized. Our rule has the following behavior for
each argument:

- `_: User`: the `actor` argument must have the `User` type.
- `"create"`: the `action` must match the string literal `"create"`
- `_: Org`: the `resource` argument must have the `Org` type.

Notice that `User` and `Org` are SQLAlchemy models. We can reference
these in our policy because we registered them with `register_models`!

## Performing authorization

To authorize a request, we use the `Oso.is_allowed` method. Here's an
example in our Org creation handler:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes.py"
    from="docs: begin-is-allowed"
    to="docs: end-is-allowed"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

## Session Setup

Oso integrates with SQLAlchemy sessions to authorize access to models.
In a typical application, we may have one SQLAlchemy session per
request. Often this is accomplished with a session factory that is
scoped to the [current
thread](https://docs.sqlalchemy.org/en/13/orm/contextual.html).

When using Oso, two SQLAlchemy sessions are usually necessary:

- *authorized session*: A session instance that will only return
  authorized objects from a query. An authorized session is fixed to one
  *authorization query* (that is one set of `actor`, `action`, and resource
  type). *An authorized session only applies authorization on read
  queries*. To authorize mutations to single objects, use `is_allowed`.
- *basic session*: A session instance that returns all data, including
  data not authorized for the current user.

The authorized session is used for fetching data from the database that
must be limited to the current user. When performing authorization, Oso
uses [data filtering](../../guides/data_access) to translate the
policy's rules into a SQLAlchemy query. Only authorized objects will be
fetched from the database.

The basic session should be used for queries that should not have
authorization applied. Often fetching the current user will use the
basic session.

We can create these sessions using
`sqlalchemy_oso.authorized_sessionmaker`.

**TODO code**

In our Flask application, we create these session in the before request
hook. This gives us a convenient spot to do session initialization.

For more on session management with SQLAlchemy, see **TODO** [here]().

## Controlling access with roles

Now, let's write our first rules that use role based access control. To
setup the role library, we must:

1. Initialize `OsoRoles`
2. Add role and resource configurations to our policy.
3. Use the `Roles.role_allows` method in our policy.
4. Assign roles to users.
5. Configure `OsoRoles` to persist role configuration.

### Initializing `OsoRoles`

`OsoRoles` extends Oso with role specific configuration & enforcement.
We enable it by constructing the `OsoRoles` object and calling `enable`.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/__init__.py"
    from="docs: begin-init-oso"
    to="docs: end-init-oso"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    hlOpts="hl_lines=7-8"
    >}}

### Configuring our first resource

Roles in Oso are scoped to resources. A role is a grouping of
permissions that may be performed on that resource. Roles are assigned
to actors to grant them all permissions the role has.

We define resources in Polar using the ``resource`` rule. The `Org`
resource represents an Organization in the GitClub example application.
Let's walk through the resource definition for `Org`.

```polar
resource(_type: Org, "org", actions, roles) if
```

The rule head has 4 parameters:

- `_type` is the SQLAlchemy model the role is associated with.
- `"org"` is the identifier for this resource type. (this can be any string
you choose)
- `actions`: Are set in the rule body. The `actions` list defines all
  actions that may be performed on the resource.
- `roles`: Are set in the rule body. The `roles` dictionary defines all
  roles for this resource.

In our rule body, we first define the list of actions:

```polar
resource(_type: Org, "org", actions, roles) if
    actions = ["read", "create_repo"] and
    roles = {
        ...
    };
```

Each request authorized by Oso has an associated action and resource
type.

Now, let's define our roles. Roles are defined in a dictionary that maps
the role name to a list of permissions and (optionally) implied roles.
We'll cover implied roles later.

```polar
resource(_type: Org, "org", actions, roles) if
    actions = ["read", "create_repo"] and
    roles = {
        org_member: {
            perms: ["read"],
        },
        org_owner: {
            perms: ["read", "create_repo"],
        }
    };
```

This resource definition defines two roles:

- *org_member*: Has the `read` permission.
- *org_owner*: Has the `read` and `create_repo` permissions.

Permissions are actions associated with a resource. A permission can
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
        org_member: {
            perms: ["read"],
        },
        org_owner: {
            perms: ["read", "create_repo"],
        }
    }
);
```

But, we think the expanded form is clearer.

{{% /callout %}}

### Adding role_allows

To allow access based on roles, we call `Roles.role_allow` in our
policy:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-role-allow"
    to="docs: end-role-allow"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

Oso will now allow access to any resource that is allowed based on the
role definitions.

### Assigning roles to users

Now we've configured roles and setup our policy. Finally, we must assign
users to roles.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes.py"
    from="docs: begin-role-assignment"
    to="docs: end-role-assignment"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    >}}

**TODO** api docs

The `assign_role` function assigns a particular role on a resource.
Often, you'll make this available in a route so that end users can
configure role assignments for their organization. More on this in [end
user configuration](end_user_configuration).

Role assignment is stored your database along with the rest of your
SQLAlchemy managed data.

### Persisting roles configuration

Oso stores role and permission configuration in your database alongside
the rest of your SQLAlchemy tables. Additional models are added to your
`Base` class metadata when initializing `OsoRoles`.

But, we still must persist the role configuration from the policy into
the database. We do this with the `OsoRoles.configure` method. The
`configure` method will replace all role configuration to reflect the
`resource` definitions in the policy.

In our sample app, we call `configure` in initialization of our app:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/__init__.py"
    from="docs: begin-configure"
    to="docs: end-configure"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    >}}

{{% callout "Production tip" "orange" %}}

Since `configure()` performs bulk database operations, a production
application should call it as part of the deployment process in a
script.

{{% /callout %}}


{{% callout "Improved role configuration migrations coming soon" "green" %}}

Currently, `OsoRoles.configure` deletes and replaces all role
configuration in the database. In a future release, we will have a
migration tool that only synchronizes changes to the database, and warns
when removing roles or permissions that are in use.

Roles and permissions are stored in the database to allow creation of
dynamic role and permission assignments in a future release.

{{% /callout %}}

### Implying roles

The `"org_owner"` role is a more permissive role than `"org_member"`. It
covers all the permissions of `"org_member"`, with some additional
permissions granted (`"create_repo"`) in our example.

Instead of duplicating the permissions, we can represent this
relationship in our policy using **implied roles**.

```polar
resource(_type: Org, "org", actions, roles) if
    actions = ["read", "create_repo"] and
    roles = {
        org_member: {
            perms: ["read"],
        },
        org_owner: {
            perms: ["create_repo"],
            implies: ["org_member"]
        }
    };
```

The `"org_owner"` now implies the `"org_member"` role. Any user with the
`"org_owner"` role will be granted all permissions associated with both
roles.

Here's the full `Org` resource definition from the GitClub example app:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-org-resource"
    to="docs: end-org-resource"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

Notice the `"repo_read"` and `"repo_write"` implications. These are
roles defined on another resource, Repository. In the next guide, we'll
see how to setup **cross resource implied roles** like these!
