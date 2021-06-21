---
title: Add Oso to an App
weight: 2
description: |
  Adding Oso to Node.js applications.
aliases:
  - /getting-started/application/index.html
---


# Add Authorization to a Node App

After reading this guide, you will know how to:

- Add Oso to a Node application.
- Enforce authorization in a Node web app, preventing unauthorized access to
  sensitive data.
- Write fine-grained authorization rules in Polar, a declarative logic
  language.

## Getting started

To illustrate the steps of adding authorization to a Node app, we'll be
working with an example expenses-tracking application that's [available on
GitHub][example-repo]. The app uses Express, but the patterns covered in this
guide apply to any framework.

[example-repo]: https://github.com/osohq/oso-express-tutorial

Clone [the example app][example-repo], install dependencies in a virtual
environment, seed the database, and fire up the server:

```console
$ git clone https://github.com/osohq/oso-express-tutorial.git
$ cd oso-express-tutorial
$ npm install
$ sqlite3 expenses.db ".read expenses.sql"
$ npm run start
```

To verify that everything's set up correctly, open a new terminal and make a
request:

```console
$ curl -H "user: alice@foo.com" localhost:3000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

Right now, the app has no authorization in place; *anyone* can access *any*
expense. This is very bad! An expense might contain private information from
the person who submitted it, and we don't want to make that information public.
Adding authorization — in this case, limiting which expenses a user is allowed
to see — ensures we don't leak private data.

To start adding authorization to the app, let's set up Oso.

## Adding Oso

First, kill the running Node server, and install the Oso library:

```console
$ npm install oso
+ oso@{{< version >}} ...
```

Once the library's installed, create a new file in the `src` directory called
`authorization.ts`. In this file, we'll write helper functions for
initializing and accessing Oso:

{{< literalInclude
    path="examples/node/getting-started/application/src/authorization.ts"
    lines="1-4,6-12,14-20" >}}

We've **(1)** imported the `Oso` module, **(2)** constructed a new Oso instance,
**(3)** registered a pair of our application classes with Oso so that we can
reference them in our to-be-written authorization policy, and **(4)** exported a
function to initialize Oso or return a cached reference to the instance.

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

Back in `src/authorization.ts`, let's create another helper function:

{{< literalInclude
    path="examples/node/getting-started/application/src/authorization.ts"
    from="// start-authorize" >}}

We're only securing a single controller method in this guide, but it's still a
good idea to encapsulate this authorization logic for future reuse and to keep
it separate from the app's business logic.

Let's use the new helper function to apply authorization in our `getExpense` 
controller function:

{{< literalInclude
    path="examples/node/getting-started/application/src/controllers.ts"
    lines="3,5-16"
    hlOpts="hl_lines=10" >}}

Restart the Express app, and then repeat the same request from earlier. It should
now result in a `403 Forbidden`:

```console
$ curl -H "user: alice@foo.com" localhost:3000/expenses/1
Forbidden
```

Oso is deny-by-default. Since we haven't given Oso any rules allowing access,
*every* request handled by the `/expenses/:expenseId` handler will currently be 
denied. We've certainly prevented unauthorized access to expense data, but we've 
gone a bit too far. In the next section, we'll learn how to write fine-grained 
rules to enable users to view only the expenses they should have access to.

## Writing fine-grained authorization rules

In this final section, we're going to write an authorization policy that allows
users to view certain expenses that they should have access to. Oso understands
policies written in [Polar](learn/polar-foundations), our declarative language
for expressing authorization logic.

In the `src` directory, create a new file named `authorization.polar`. We're
going to load that file into Oso in the `initOso()` function we created
earlier:

{{< literalInclude
    path="examples/node/getting-started/application/src/authorization.ts"
    lines="8,015"
    hlOpts="hl_lines=13" >}}

At this point, all requests to `getExpense` will still be denied because our
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
    path="examples/node/getting-started/application/src/authorization.polar"
    lines="1-2" >}}

Add that rule to `src/authorization.polar`, restart the server, and the same
request should once again succeed since `alice@foo.com` submitted the `Expense`
with `id=2`:

```console
$ curl -H "user: alice@foo.com" localhost:3000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

If we try the same request as a different user, Oso prevents us from accessing
`alice@foo.com`'s expense:

```console
$ curl -H "user: bhavik@foo.com" localhost:3000/expenses/2
Forbidden
```

{{% callout "Note" "blue" %}}
  For more details on Polar language syntax, refer to the [Polar syntax
  guide](polar-syntax).
{{% /callout %}}

### A more complex example: composing authorization rules

Our example was quick to set up, but we also could have gotten the same result with a 
Javascript `if` statement. Polar shines when composing more complex rules that would 
otherwise be difficult conditionals. Let's add a twist to our authorization rule.

{{% callout "Our Goal" "green" %}}
A user is allowed to view any expense if they are an accountant.
{{% /callout %}}

Here, we'll add the concept of a *role*, like `accountant`.
In this case, a user has the role of `accountant` if their job title is "Accountant".


{{< literalInclude
    path="examples/node/getting-started/application/src/authorization.polar"
    lines="4-5" >}}

Here's one place Polar comes in handy: we can add extra information about roles ad hoc.
Senior accountants are also accountants.

{{< literalInclude
    path="examples/node/getting-started/application/src/authorization.polar"
    lines="7-8" >}}

This looks like a re-definition of `user_in_role`, but to Polar, this is adding more information.
In English, you can read these Polar statements as:

- "It is true that a user is an `accountant` if their title is 'Accountant'."
- "It is true that a user is an `accountant` if their title is 'Senior Accountant'."

We could even use this to add information about other roles, like `admin`s or `manager`s.

Now, we can add an `allow` statement to check if a user has the correct role:

{{< literalInclude 
  path="examples/node/getting-started/application/src/authorization.polar" 
  lines="10-11" >}}

The user with the email `bhavik@foo.com` is a Senior Accountant, so they can now access Alice's expense!

```console
$ curl -H "user: bhavik@foo.com" localhost:3000/expenses/2
Expense(amount=17743, description='Pug irony.', user_id=1, id=2)
```

{{% callout "What's next" "blue" %}}

<!-- TODO(gj): page doesn't exist yet in new docs
- To explore integrating Oso in your app in more depth continue to [Access Patterns](). -->
- To learn about different patterns for structuring authorization code, see
  [Role-Based Access Control (RBAC) Patterns](learn/roles).
- For a deeper introduction to policy syntax, see [Writing Policies](policies).
- For reference on using the Node Oso library, see [Node Authorization Library](reference).

Specific tutorials on integrating
you may find some of our blog posts useful, especially
and [GraphQL Authorization with Graphene, SQLAlchemy and Oso](https://www.osohq.com/post/graphql-authorization-graphene-sqlalchemy-oso).
Please also see the reference pages on [Framework & ORM Integrations](reference/frameworks).

Want to know even more about adding Oso to your Node app? Here are some other great resources that might help you get started.

## NestJS library by Bjerk AS

The good folks at [Bjerk AS][bjerk] have made a [NestJS][nestjs] integration
for Oso available via [GitHub][nestjs-oso-github] and [npm][nestjs-oso-npm].
Its documentation includes a Quickstart guide.

## Sam's blog post

Oso CTO Sam Scott blogged about [Adding Authorization to a Node.js
App][adding-authorization-post]. That post also uses [NestJS][nestjs] as a
framework, but without the adapter library above. It also explores some common
authorization patterns and how to express them in that framework.

[adding-authorization-post]: https://www.osohq.com/post/adding-authorization-nodejs-app-beyond-role-based-access-control
[bjerk]: https://bjerk.io/
[nestjs]: https://nestjs.com/
[nestjs-oso-github]: https://github.com/bjerkio/nestjs-oso#readme
[nestjs-oso-npm]: https://www.npmjs.com/package/nestjs-oso

{{% /callout %}}
