---
title: Roles with SQLAlchemy
weight: 1
---

The `sqlalchemy_oso.roles` module provides out-of-the-box Role-Based Access Control features that
let you create a roles system  with a few lines of code, and specify role permissions
in a declarative oso policy.

This guide walks you through how to use `sqlalchemy_oso` to add basic roles to a multi-tenant app.

{{< callout "Note" "green" >}}
We’re using a Flask app for this example, but the
`sqlalchemy_oso` library can be used with any Python application.
{{< /callout >}}

## 1. Set up the application

### Install the oso SQLAlchemy package

Install the `sqlalchemy_oso` package.

```
$ pip install sqlalchemy_oso
```

Alternatively, if you are starting from scratch, clone the [sample
application](https://github.com/osohq/oso-sqlalchemy-roles-guide/tree/main)
and use the provided `requirements.txt` file.

```
$ pip install -r requirements.txt
```

Add a method to initialize oso and make the oso instance available to your
application code. This method should initialize oso and load your policy file
(can be an empty `.polar` file). It should also call
`sqlalchemy_oso.session.set_get_session()` to configure access to the
SQLALchemy session oso should use to make queries. Then call
`sqlalchemy_oso.roles.enable_roles()` to load the base oso policy for
roles.

### Create a users model

Add a user model that will represent your app’s users (if you don’t already have one).

### Create an organizations model

Add an organization model that will represent the organizations or tenants
that users belong to. The roles you create will be scoped to this model.

### Add an endpoint that needs authorization

Create or choose an existing endpoint that will need authorization to access.
In our sample app we’ve created two endpoints that have different authorization
requirements: one to view repositories and another to view billing
information.

Add policy checks to your code to control access to the protected endpoints.

Our example uses `flask_oso.FlaskOso.authorize()` to complete the
policy check, which returns a `403 Forbidden` response if the provided
`actor` is not allowed to take `action` on the `resource` passed as the
first argument. If you’re not using Flask, you can use
`oso.Oso.is_allowed()` from our general-purpose Python package.

Since we haven’t added any rules to our policy file yet, these endpoints will
return a `403 Forbidden` response to all requests.

## 2. Add roles

### Create the OrganizationRole class using the role mixin

The oso SQLAlchemy library provides the
`sqlalchemy_oso.roles.resource_role_class()` method to generate a
mixin which creates a role model. Create the mixin by passing in the base,
user, and organization models, as well as the role names. Then create a role
model that extends it.

### Specify role permissions

To give the roles permissions, write an oso policy.

Since we already called `sqlalchemy_oso.roles.enable_roles()` in our `init_oso()` method,
you can write Polar `role_allow` rules over `OrganizationRoles`.

You can also specify a hierarchical role ordering with `organization_role_order` rules.

For more details on the roles base policy, see Built-in Role-Based Access Control.

### Create an endpoint for assigning roles

Until you assign users to roles, they’ll receive a `403 FORBIDDEN` response if they try to
access either protected endpoint.

Add a new endpoint to your application that users can hit to assign roles. To
control who can assign roles, add another call to
`flask_oso.FlaskOso.authorize()`.

Use the oso role API to create role assignments with
`sqlalchemy_oso.roles.add_user_role()` and
`sqlalchemy_oso.roles.reassign_user_role()`.

### Configure permissions for role assignments

Update the oso policy to specify who is allowed to assign roles.

## 3. Test it works

### Run the application

Start the server.

```
$ flask run
* Running on http://127.0.0.1:5000/ (Press CTRL+C to quit)
```

Make a simple request.

```
$ curl --header "user: ringo@beatles.com" localhost:5000/
Hello ringo@beatles.com
```

### Try it out

Try to access the protected endpoints. Access should be granted or denied
based on your policy. Our sample app includes some fixture data for testing.
To run the server with fixture data, set the `FLASK_APP` environment
variable.

```
$ export FLASK_APP="app:create_app(None, True)"
$ flask run
* Running on http://127.0.0.1:5000/ (Press CTRL+C to quit)
```

Our policy says that users with the
“OWNER” role can assign roles, users with the “MEMBER” role can view
repositories, and users with the “BILLING” role can view billing info. Also, the
“OWNER” roles inherits the permissions of the “MEMBER” and “BILLING” roles.

Paul is a member of “The Beatles” organization, so he can view repositories but not
billing info:

```
$ curl --header "user: paul@beatles.com" localhost:5000/orgs/1/repos
{"repos":[{"id":1,"name":"Abbey Road"}]}

$ curl --header "user: paul@beatles.com" localhost:5000/orgs/1/billing
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Unauthorized</p>
```

John is the owner of “The Beatles” so he can assign roles:

```
$ curl --header "Content-Type: application/json" \
--header "user: john@beatles.com"  \
--request POST \
--data '{"name":"BILLING", "user_email":"ringo@beatles.com"}' \
http://localhost:5000/orgs/1/roles
created a new role for org: 1, ringo@beatles.com, BILLING
```

But Ringo isn’t an owner, so his access should be denied:

```
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

The fully-implemented GitHub sample app, complete with tests, can be found [here](https://github.com/osohq/oso-sqlalchemy-roles-guide/tree/basic_roles_complete).
