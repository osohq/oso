---
title: Add Roles with SQLAlchemy
description: Learn how to use Oso's SQLAlchemy integration to add roles to a multi-tenant app.
weight: 2
layout: list
_build:
  list: never
---

# Add Roles with SQLAlchemy

{{% callout "Early access to the next version of sqlalchemy-oso" "blue" %}}

We just released early access to the next version of our SQLAlchemy
library which includes improved role configuration and enforcement.

[Check it out here!](/new-roles)

{{% /callout %}}

The [`sqlalchemy_oso.roles`](reference/frameworks/sqlalchemy#sqlalchemy_osoroles) module
provides out-of-the-box Role-Based Access Control features that let you create
a roles system with a few lines of code and specify role permissions in a
declarative Oso policy.

This guide walks you through how to use `sqlalchemy_oso` to add basic roles to a multi-tenant app.

{{% callout "Note" "green" %}}
We’re using a Flask app for this example, but the
`sqlalchemy_oso` library can be used with any Python application.
{{% /callout %}}

## 1. Set up the application

### Install the Oso SQLAlchemy package

Install the `sqlalchemy_oso` package.

```console
$ pip install sqlalchemy_oso
```

Alternatively, if you are starting from scratch, clone the [sample
application](https://github.com/osohq/oso-sqlalchemy-roles-guide) and
use the provided `requirements.txt` file:

```console
$ pip install -r requirements.txt
```

Add a method to initialize Oso and make the Oso instance available to your
application code. This method should initialize Oso and load your policy file,
which can be an empty `.polar` file. It should also call
`sqlalchemy_oso.session.set_get_session()` to configure access to the
SQLAlchemy session Oso should use to make queries. Then call
`sqlalchemy_oso.roles.enable_roles()` to load the base Oso policy for roles:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/__init__.py"
                   lines="6,11-14,57-66" >}}

### Create a users model

Add a `User` model that will represent your app’s users (if you don’t already
have one):

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/models.py"
                   lines="9,14-21" >}}

### Create an organizations model

Add an organization model that will represent the organizations or tenants that
users belong to. The roles you create will be scoped to this model:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/models.py"
                   lines="24-32" >}}

### Add an endpoint that needs authorization

Create or choose an existing endpoint that will need authorization to access.
In our sample app we’ve created two endpoints that have different authorization
requirements: one to view repositories and another to view billing information.

Add policy checks to your code to control access to the protected endpoints:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/routes.py"
                   lines="1-2,6,14-27" >}}

Our example uses `flask_oso.FlaskOso.authorize()` to complete the policy check,
which returns a `403 Forbidden` response if the provided `actor` is not allowed
to take `action` on the `resource` passed as the first argument. If you’re not
using Flask, you can use `oso.Oso.is_allowed()` from our general-purpose
[Python package](reference).

Since we haven’t added any rules to our policy file yet, these endpoints will
return a `403 Forbidden` response to all requests.

## 2. Add roles

### Create the `OrganizationRole` class using the role mixin

The Oso SQLAlchemy library provides the
`sqlalchemy_oso.roles.resource_role_class()` method to generate a mixin which
creates a role model. Create the mixin by passing in the base, user, and
organization models, as well as the role names. Then create a role model that
extends it:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/models.py"
                   lines="6,51-58" >}}

### Specify role permissions

To give the roles permissions, write an Oso policy.

Since we already called `sqlalchemy_oso.roles.enable_roles()` in our
`init_oso()` method, you can write Polar `role_allow` rules over
`OrganizationRole`s:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/authorization.polar"
                   lines="7-14" >}}

You can also specify a [hierarchical role
ordering](learn/roles#role-hierarchies) with `organization_role_order`
rules:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/authorization.polar"
                   lines="24-26" >}}

For more details on the roles base policy, see [Built-in Role-Based Access
Control](learn/roles).

### Create an endpoint for assigning roles

Until you assign users to roles, they’ll receive a `403 FORBIDDEN` response if
they try to access either protected endpoint.

Next, add a new endpoint to your application that users can hit to assign
roles. To control who can assign roles, add another call to
`flask_oso.FlaskOso.authorize()`. Additionally, use the Oso role API to create
role assignments with `sqlalchemy_oso.roles.add_user_role()` and
`sqlalchemy_oso.roles.reassign_user_role()`:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/routes.py"
                   lines="30-47"
                   hlOpts="hl_lines=4 13 16" >}}

### Configure permissions for role assignments

Update the Oso policy to specify who is allowed to assign roles:

{{< literalInclude path="examples/roles/sqlalchemy/basic/app/authorization.polar"
                   lines="16-17" >}}

## 3. Test it works

### Run the application

Start the server:

```console
$ flask run
* Running on http://127.0.0.1:5000/ (Press CTRL+C to quit)
```

Make a simple request:

```console
$ curl --header "user: ringo@beatles.com" localhost:5000/
Hello ringo@beatles.com
```

### Try it out

Try to access the protected endpoints. Access should be granted or denied based
on your policy. Our sample app includes some fixture data for testing. To run
the server with fixture data, set the `FLASK_APP` environment variable:

```console
$ export FLASK_APP="app:create_app(None, True)"
$ flask run
* Running on http://127.0.0.1:5000/ (Press CTRL+C to quit)
```

Our policy says that users with the “OWNER” role can assign roles, users with
the “MEMBER” role can view repositories, and users with the “BILLING” role can
view billing info. Also, the “OWNER” roles inherits the permissions of the
“MEMBER” and “BILLING” roles.

Paul is a member of “The Beatles” organization, so he can view repositories but
not billing info:

```console
$ curl --header "user: paul@beatles.com" localhost:5000/orgs/1/repos
{"repos":[{"id":1,"name":"Abbey Road"}]}

$ curl --header "user: paul@beatles.com" localhost:5000/orgs/1/billing
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Unauthorized</p>
```

John is the owner of “The Beatles” so he can assign roles:

```console
$ curl --header "Content-Type: application/json" \
--header "user: john@beatles.com"  \
--request POST \
--data '{"name":"BILLING", "user_email":"ringo@beatles.com"}' \
http://localhost:5000/orgs/1/roles
created a new role for org: 1, ringo@beatles.com, BILLING
```

But Ringo isn’t an owner, so his access should be denied:

```console
$ curl --header "Content-Type: application/json" \
--header "user: ringo@beatles.com"  \
--request POST \
--data '{"name":"BILLING", "user_email":"ringo@beatles.com"}' \
http://localhost:5000/orgs/1/roles
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Unauthorized</p>
```

The fully-implemented GitHub sample app, complete with tests, can be found
[here](https://github.com/osohq/oso-sqlalchemy-roles-guide/tree/basic_roles_complete).

{{% callout "What's next" "blue" %}}

- Read more about using [roles](learn/roles) with Oso.

{{% /callout %}}
