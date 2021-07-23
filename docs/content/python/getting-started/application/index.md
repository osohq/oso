---
title: Add Oso to an App (15 min)
weight: 2
description: |
  An in-depth walkthrough of adding Oso to an example expense application.
aliases:
  - /getting-started/application/index.html
---

# Add Authorization to a Python App

After reading this guide, you will know how to:

- Add Oso to a Python application.
- Enforce authorization in a Python web app, preventing unauthorized access to
  sensitive data.
- Write fine-grained authorization rules in Polar, a declarative logic
  language.

## Getting started

To illustrate the steps of adding authorization to a Python app, we'll be
working with an example expenses-tracking application that's [available on
GitHub][example-repo]. The app uses Flask, but the patterns covered in this
guide apply to any framework.

[example-repo]: https://github.com/osohq/oso-flask-tutorial

Clone [the example app][example-repo], install dependencies in a virtual
environment, seed the database, and fire up the server:

```console
$ git clone https://github.com/osohq/oso-flask-tutorial.git
$ cd oso-flask-tutorial
$ python3 -m venv venv && source venv/bin/activate
$ pip3 install -r requirements.txt
$ sqlite3 expenses.db ".read expenses.sql"
$ FLASK_ENV=development flask run
```

To verify that everything's set up correctly, open a new terminal and make a
request:

```console
$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

Right now, the app has no authorization in place; *anyone* can access *any*
expense. This is very bad! An expense might contain private information from
the person who submitted it, and we don't want to make that information public.
Adding authorization — in this case, limiting which expenses a user is allowed
to see — ensures we don't leak private data.

To start adding authorization to the app, let's set up Oso.

## Adding Oso

First, kill the running Flask server, and install the Oso library:

```console
$ pip3 install --upgrade oso
Successfully installed oso-{{< version >}}
```

Once the library's installed, create a new file in the `app` directory called
`authorization.py`. In this file, we'll write a helper function for
initializing Oso:

{{< literalInclude
    path="examples/python/getting-started/application/app/authorization.py"
    to="oso.load_file(\"app/authorization.polar\")" >}}

We've **(1)** imported the `Oso` class, **(2)** constructed a new Oso instance,
**(3)** registered a pair of our application classes with Oso so that we can
reference them in our to-be-written authorization policy, and **(4)** attached
the Oso instance to the passed-in Flask application, making it accessible
throughout our app.

We'll call our new helper function during application setup:

{{< literalInclude
    path="examples/python/getting-started/application/app/__init__.py"
    lines="6-9,21-23" >}}

With Oso set up, let's start enforcing authorization to protect our application
data.

## Enforcing authorization

There are several potential places to enforce authorization in a web app, from
higher-level route checks to lower-level controller or database checks.
Choosing where to enforce is a complicated topic that we covered in great
detail in [the second chapter of Authorization Academy][authz-academy].

[authz-academy]: https://www.osohq.com/academy/chapter-2-architecture

To protect our sensitive `Expense` data, we're going to enforce authorization
at the controller layer.

{{% callout "Note" "blue" %}}
  Controller-level authorization is a very common pattern in web apps because
  of the rich authorization context available at that point in the request
  lifecycle.
{{% /callout %}}

Back in `app/authorization.py`, let's create another helper function:

{{< literalInclude
    path="examples/python/getting-started/application/app/authorization.py"
    from="# start-authorize" >}}

We're only securing a single controller method in this guide, but it's still a
good idea to encapsulate this authorization logic for future reuse and to keep
it separate from the app's business logic.

Let's use the new helper function to apply authorization in our `get_expense()`
controller method:

{{< literalInclude
    path="examples/python/getting-started/application/app/expense.py"
    lines="8,55-58"
    hlOpts="hl_lines=8" >}}

Restart the Flask app, and then repeat the same request from earlier. It should
now result in a `403 Forbidden`:

```console
$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>
```

Oso is deny-by-default. Since we haven't given Oso any rules allowing access,
*every* request handled by the `get_expense()` method will currently be denied.
We've certainly prevented unauthorized access to expense data, but we've gone a
bit too far. In the next section, we'll learn how to write fine-grained rules
to enable users to view only the expenses they should have access to.

## Writing fine-grained authorization rules

In this final section, we're going to write an authorization policy that allows
users to view certain expenses that they should have access to. Oso understands
policies written in [Polar](learn/polar-foundations), our declarative language
for expressing authorization logic.

In the `app` directory, create a new file named `authorization.polar`. We're
going to load that file into Oso in the `init_oso()` function we created
earlier:

{{< literalInclude
    path="examples/python/getting-started/application/app/authorization.py"
    lines="1,9-13"
    hlOpts="hl_lines=9" >}}

At this point, all requests to `get_expense()` will still be denied because our
policy is empty.

{{% callout "Note" "blue" %}}
  When starting out, it's fine to store all policy logic in a single Polar
  file. As the policy grows, it's natural to break it out into multiple Polar
  files to keep everything organized.
{{% /callout %}}

### Users can view their own expenses

The first authorization rule we're going to enforce is that **a user should be
allowed to view an expense if they submitted it**.

Our `Expense` class has a `user_id` field that stores the ID of the submitting
`User`. We can encode the desired logic in Polar as follows:

{{< literalInclude
    path="examples/python/getting-started/application/app/authorization.polar"
    lines="1-2" >}}

Add that rule to `app/authorization.polar`, restart the server, and the same
request should once again succeed since `alice@foo.com` submitted the `Expense`
with `id=2`:

```console
$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

If we try the same request as a different user, Oso prevents us from accessing
`alice@foo.com`'s expense:

```console
$ curl -H "user: bhavik@foo.com" localhost:5000/expenses/2
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>
```

{{% callout "Note" "blue" %}}
  For more details on Polar language syntax, refer to the [Polar syntax
  guide](polar-syntax).
{{% /callout %}}

### A more complex example: composing authorization rules

Our example was quick to set up, but we also could have gotten the same result with a Python `if` statement.
Polar shines when composing more complex rules that would otherwise be difficult conditionals.
Let's add a twist to our authorization rule.

{{% callout "Our Goal" "green" %}}
A user is allowed to view any expense if they are an accountant.
{{% /callout %}}

Here, we'll add the concept of a *role*, like `accountant`.
In this case, a user has the role of `accountant` if their job title is "Accountant".


{{< literalInclude
    path="examples/python/getting-started/application/app/authorization.polar"
    lines="4-5" >}}

Here's one place Polar comes in handy: we can add extra information about roles ad hoc.
Senior accountants are also accountants.

{{< literalInclude
    path="examples/python/getting-started/application/app/authorization.polar"
    lines="7-8" >}}

This looks like a re-definition of `user_in_role`, but to Polar, this is adding more information.
In English, you can read these Polar statements as:

- "It is true that a user is an `accountant` if their title is 'Accountant'."
- "It is true that a user is an `accountant` if their title is 'Senior Accountant'."

We could even use this to add information about other roles, like `admin`s or `manager`s.

Now, we can add an `allow` statement to check if a user has the correct role:

{{< literalInclude path="examples/python/getting-started/application/app/authorization.polar" lines="10-11">}}

The user with the email `bhavik@foo.com` is a Senior Accountant, so they can now access Alice's expense!

```console
$ curl -H "user: bhavik@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

{{% callout "What's next" "blue" %}}

<!-- TODO(gj): page doesn't exist yet in new docs
- To explore integrating Oso in your app in more depth continue to [Access Patterns](). -->
- To learn about different patterns for structuring authorization code, see
  [Role-Based Access Control (RBAC) Patterns](learn/roles).
- For a deeper introduction to policy syntax, see [Writing Policies](policies).
- For reference on using the Python Oso library, see [Python Authorization Library](reference).

Specific tutorials on integrating
Oso with other common Python frameworks are coming soon, but in the meantime
you may find some of our blog posts useful, especially
[Building a Django app with data access control in 30 minutes](https://www.osohq.com/post/django-access-control)
and [GraphQL Authorization with Graphene, SQLAlchemy and Oso](https://www.osohq.com/post/graphql-authorization-graphene-sqlalchemy-oso).
Please also see the reference pages on [Framework & ORM Integrations](reference/frameworks).

{{% /callout %}}
