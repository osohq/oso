---
title: Getting started
weight: 2
description: >
    Getting started with SQLAlchemy roles
---

**TODO** i want every API link to go to reference

**TODO** i want code snippets to be full paths

**TODO** i want to configure literal include in the front matter so i
don't need params each time

# Getting started

## Installation

## Setup

The SQLAlchemy roles library extends the ``Oso`` core library with built
in configuration, data model and enforcement of roles based access
control.

In this guide, we'll walk through the basics of starting to use the
SQLAlchemy roles library, using the **TODO** [GitClub]() application as an
example.

## Oso Instance Setup

The ``Oso`` class is the entrypoint to using Oso in our application. We
usually will have a global instance that is created during application
initialization, and shared across requests. In a flask application, you
may attach this instance to the global flask object, or the current
application.

**CODE SAMPLE**

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/__init__.py"
    lines="108-117"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
>}}

## Loading our policy

`Oso` uses the Polar language to define authorization policies. An
authorization policy specifies what requests are allowed. The entrypoint
to a policy is the `allow` rule, which specifies when a request for a
given `actor`, `action` and `resource` is allowed. For more on
policy basics see our **TODO** [super great policy guide]().

We can write simple rules in our policy, like this one which allows all
users to create new organizations:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    lines="10"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
>}}

**TODO** link to API documentation

This rule allows all requests. A policy can be loaded into ``Oso`` using
the `Oso.load_file` function.

```polar
oso.load_file("policy.polar")
```

## SQLAlchemy Setup

We will be using Oso with SQLAlchemy. As part of this process, we need
to make Oso aware of our SQLAlchemy models. Later, we'll write
authorization rules to control access to our models.

The `sqlalchemy_oso.register_models` function helps us register
SQLAlchemy ORM models with Oso.

```polar
register_models(Base, oso)
```

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
  queries*. To authorize mutations, use `is_allowed` *TODO* link.
- *basic session*: A session instance that returns all data, including
  data not authorized for the current user.

The authorized session is used for fetching data from the database that
must be limited to the current user. When performing authorization, Oso
uses [data filtering]() **TODO link** to translate the policy's rules into a
SQLAlchemy query. Only authorized objects will be fetched from the
database.

The basic session should be used for queries that should not have
authorization applied. Often fetching the current user will use the
basic session.

**TODO** whatever the app uses write here.

We can create these sessions using
`sqlalchemy_oso.authorized_sessionmaker`.

In our Flask application, we create these session in the before request
hook. This gives us a convenient spot to do session initialization.

For more on session management with SQLAlchemy, see **TODO** [here]().

## Writing an authorization rule over a SQLAlchemy model

We can write rules in our Polar policy to control access to SQLAlchemy
models.

```polar
allow(_user: User, "read", repository: Repository) if
    repository.is_public;
```

This rule allows a user to perform the `"read"` action on a `repository`
if the repository `is_public` field is `true`. A Polar rule has a head
(before `if`) that matches authorization parameters by value or
type. The body (after `if`) specifies conditions that must be satisfied
for the request to be authorized.

## Controlling access with organization roles

Now that we've written a simple rule, let's start adding some role based
access control to our policies.

Let's begin by adding some built-in roles to our app at the organization
level. This means that these roles will control access within a single
organization, but won't provide fine-grained access control over
resources within an organization (e.g., repository-level access
control).

First, set up Oso:

```python
# Set up oso
oso = Oso()
oso.register_class(User)
oso.register_class(Organization)

# Set up roles
roles = OsoRoles(oso)
roles.enable()
```

Now, let's configure our organization roles in our Polar policy:

```polar
# Define Organization roles
resource(_type: Organization, "org", actions, roles) if
    actions = [     # the actions that exist for Organizations
        "invite",
        "create_repo"
    ] and
    roles = {       # the roles that exist for organizations
        org_member: {
            perms: ["create_repo"]  # role-permission assignments
        },
        org_owner: {
            perms: ["invite"]
        }
    };

# Use roles to evaluate allow queries
allow(actor, action, resource) if
    Roles.role_allows(actor, action, resource);
```

After loading the policy, we can assign users to roles:

```python
# Load the policy file
oso.load_str("policy.polar")

# Demo data
osohq = Organization(id="osohq")

leina = User(name="Leina")
steve = User(name="Steve")

# Assign users to roles
roles.assign_role(leina, osohq, "org_owner")
roles.assign_role(steve, osohq, "org_member")
```

Let's write a few tests to show that the roles are working:

```python
# Leina can invite people to osohq because she is an OWNER
assert oso.is_allowed(leina, "invite", osohq)

# Steve can create repos in osohq because he is a MEMBER
assert oso.is_allowed(steve, "create_repo", osohq)

# Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
assert not oso.is_allowed(steve, "invite", osohq)
```
