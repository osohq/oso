---
title: Add Oso to an App (15 min)
weight: 2
description: |
  An in-depth walkthrough of adding Oso to an example expense application.
aliases:
  - /getting-started/application/index.html
---

# Add Oso to a Python Application

## Our Sample App

For this guide, we've written a sample expenses-tracking app.
Right now, it has no authorization policies at all.
Anyone can access any expense, which is bad!
An expense might contain private information from the person who submitted it, and we'd like to avoid making that information public.

Adding authorization — in this case, limiting what data a user can see — will let us make sure we don't leak private data.

In this guide, we'll show you how to design and add authorization to this program with Oso.

Our sample expenses application is built with Flask, but the patterns we cover here can be used with any framework.

If you'd like to follow along with this tutorial, the application we're working with can be found here:

* [osohq/oso-flask-tutorial](https://github.com/osohq/oso-flask-tutorial)

Our expenses application reads from a SQLite database and has a few simple endpoints for returning results.
We encourage you to take a look around before continuing!

## Running The Example

The example application has a few requirements.
We'll install those in a virtual environment.

```console
$ git clone https://github.com/osohq/oso-flask-tutorial/
$ cd oso-flask-tutorial/
$ python3 -m venv venv
$ source venv/bin/activate
$ pip3 install -r requirements.txt
$ FLASK_ENV=development flask run --extra-files app/authorization.polar
```

The example comes with some example data, which you can load with:

```console
$ sqlite3 expenses.db ".read expenses.sql"
```

Here's the authorization rule we'd like to enforce in our app:

{{% callout "Our Goal" "green" %}}
A user is allowed to view an expense if they submitted that expense.
{{% /callout %}}

We'll write this rule in Oso's declarative policy language, Polar.

## Adding Oso

Before we can write our authorization logic, we'll need to add Oso to our app.

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.py" lines="25-29" >}}

We can `register` our classes with Oso to be able to access their properties in our Polar code.
We'll pass the classes we're using, `User` and `Expense`.

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.py" lines="25-31" >}}

Then, we'll load our Polar file, where our logic will be.
Right now, this `app/authorization.polar` file is empty.

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.py" lines="25-33" >}}

Finally, we'll add a property `oso` to our app.
This isn't strictly necessary to start Oso.
However, we find that this is the easiest way to reference our Oso object!
Any time we have our `app` object at hand, we'll be able to perform authorization.

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.py" lines="25-34" >}}

## Calling Oso to see if an action is authorized

Oso's `is_allowed` function will let us query the authorization rules we write.

```
oso.is_allowed(user, action, resource):
```

In this case, our `action` will be "read" and our resource will be "an expense".

We'll write a helper method that we can call every time we'd like to check if a user is authorized to perform an action.

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.py" lines="17-22">}}

## Where our authorization query goes

This depends on your own app structure.
In our case, we'll put it in our Expenses controller.

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/expense.py" lines="49-52">}}

It’s easiest to apply authorization as close as possible to the resource we want to protect.
There, you'll have the most context about precisely what the user is trying to do.

## Writing our authorization policy

Now that we have Oso loaded, we can write our authorization policy!
We'll write this in the Polar language, in our `app/authorization.polar` file.

(When starting out, it's OK to capture all policy logic in a single `authorization.polar` file, like we're doing here.
As you add authorization policies for different parts of your app, you can break that up into many `.polar` files.)

Our `authorize` call will deny access to everyone.
(Oso is deny-by-default.)
Let's check that we can't access a resource that we should be able to.
(If you write your tests first, this is a great time to encode this in a test!)

In our case, we'll first query expense `2`, which was submitted by 'alice@foo.com'.

```console
$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>
```

Unfortunately, we couldn't access an expense we should have access to.
Let's fix that.

Every expense saves the `id` of the user that submitted it.
In Polar, let's check that the `id` of the current user matches the `id` of the user that created the expense. 

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.polar" lines="1-2">}}

This policy is being loaded with `oso.load_file`, and executed with `oso.is_allowed`.
Because we `register`ed our classes with `oso.register_class`, we can access those classes' properties, like `user_id`.

Let's see if it works:

```console
$ curl -H "user: alice@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

It works!
And, users that aren't `alice` can't see the expense:

```console
$ curl -H "user: bhavik@foo.com" localhost:5000/expenses/2
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<title>403 Forbidden</title>
<h1>Forbidden</h1>
<p>Not Authorized!</p>
```

This expense is secure!

## A more complex example

That was quick to set up, but we also could have gotten the same result with a Python `if` statement.
Polar shines when composing more complex rules that would otherwise be difficult conditionals.
Let's add a twist to our authorization rule.

{{% callout "Our Goal" "green" %}}
A user is allowed to view any expense if they are an accountant.
{{% /callout %}}

Here, we'll add the concept of a *role*, like `accountant`.
In this case, a user has the role of `accountant` if their job title is "Accountant".

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.polar" lines="4-5" >}}

Here's one place Polar comes in handy: we can add extra information about roles ad-hoc.
Senior accountants are also `accountants`.

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.polar" lines="7-8" >}}

This looks like a re-definition of `user_in_role`, but to Polar, this is adding more information.
In English, you can read these Polar statements as,

- "It is true that a user is an `accountant` if their title is 'Accountant'."
- "It is true that a user is an `accountant` if their title is 'Senior Accountant'."

We could even use this to add information about other roles, like `admin`s or `manager`s.

Now, we can add an `allow` statement to check if a user has the correct role:

{{< literalInclude path="examples/python/getting-started/application/expenses-flask/app/authorization.polar" lines="10-11">}}

The user with the email `bhavik@foo.com` is a Senior Accountant, so they can now access this expense!

```
$ curl -H "user: bhavik@foo.com" localhost:5000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

{{% callout "What's next" "blue" %}}

<!-- TODO(gj): page doesn't exist yet in new docs
- To explore integrating Oso in your app in more depth continue to [Access Patterns](). -->
- To learn about different patterns for structuring authorization code, see [Role-Based Authentication](https://www.osohq.com/python/learn/roles.html).
- For a deeper introduction to policy syntax, see [Writing Policies](policies).
- For reference on using the Python Oso library, see [Python Authorization Library](reference).

Specific tutorials on integrating
Oso with other common Python frameworks are coming soon, but in the meantime
you may find some of our blog posts useful, especially
[Building a Django app with data access control in 30 minutes](https://www.osohq.com/post/django-access-control)
and [GraphQL Authorization with Graphene, SQLAlchemy and Oso](https://www.osohq.com/post/graphql-authorization-graphene-sqlalchemy-oso).
Please also see our reference pages on [Framework & ORM Integrations](/reference/frameworks).

{{% /callout %}}
