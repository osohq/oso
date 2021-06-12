---
title: Quickstart (5 min)
description: |
  Ready to get started? See Oso in action, and walk through our quick
  tutorial for adding authorization to a simple web server.
weight: 1
---

# Oso in 5 minutes

Oso helps developers build authorization into their applications. If you’ve
never used Oso before and want to see it in action, this guide is for you.
We’re going to walk through how to use Oso to add authorization to a simple web
server.

{{% callout "Try it!" "green" %}}
  To follow along, clone the {{% exampleGet "githubApp" %}}:

  ```console
  git clone {{% exampleGet "githubURL" %}}
  ```
{{% /callout %}}

## Run the server

Our sample application serves data about expenses submitted by users. The
sample application has three important files.

One file defines a simple `Expense` class and some sample data stored in a map.

A second file has our HTTP server code, where we have defined a route handler
for `GET` requests to the path `/expenses/:id`. We’ve already added an
authorization check using the [Oso library](reference) to control access to
expense resources. <!-- TODO(gj): You can learn more about how to add Oso to
your application [here](Add To Your Application). -->

The third file is the Oso policy file, `expenses.polar`, and is currently
empty.

{{% callout "Try it!" "green" %}}
{{% exampleGet "installation" %}}

With the server running, open a second terminal and make a request using
cURL:

```console
$ curl localhost:5050/expenses/1
Not Authorized!
```

You’ll get a “Not Authorized!” response because we haven’t added any rules to
our Oso policy (in `expenses.polar`), and Oso is deny-by-default.
{{% /callout %}}

Let’s start implementing our access control scheme by adding some rules to the
Oso policy.

## Adding our first rule

Oso rules are written in a declarative policy language called Polar. You can
include any kind of rule in a policy, but the Oso library is designed to
evaluate [allow rules](glossary#allow-rules), which specify the conditions that
allow an **actor** to perform an **action** on a **resource**.

{{% callout "Edit it!" "blue" %}}
In our policy file (`expenses.polar`), let's add a rule that allows anyone
with an email ending in `"@example.com"` to view all expenses:

{{< literalInclude dynPath="expensesPath1"
                   fallback="expenses1" >}}

Note that the call to `{{< exampleGet "endswith" >}}` is actually calling
out to {{< exampleGet "endswithURL" >}}. The actor value passed to Oso is a
string, and Oso allows us to call methods on it.
{{% /callout %}}

The `Expense` and `String` terms following the colons in the head of the rule
are [specializers](polar-syntax#specialization), patterns that control rule
execution based on whether they match the supplied argument. This syntax
ensures that the rule will only be evaluated when the actor is a string and the
resource is an instance of the `Expense` class.

{{% callout "Try it!" "green" %}}
  Once we've added our new rule and restarted the web server, every user with
  an `@example.com` email should be allowed to view any expense:

  ```console
  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Expense(...)
  ```
{{% /callout %}}

Okay, so what just happened?

When we ask Oso for a policy decision via `Oso.{{% exampleGet "isAllowed" %}}()`, the Oso engine
searches through its knowledge base to determine whether the provided
**actor**, **action**, and **resource** satisfy any **allow** rules. In the
above case, we passed in `"alice@example.com"` as the **actor**, `"GET"` as the
**action**, and the `Expense` object with `id=1` as the **resource**. Since
`"alice@example.com"` ends with `@example.com`, our rule is satisfied, and
Alice is allowed to view the requested expense.

{{% callout "Try it!" "green" %}}
  If a user's email doesn't end in `"@example.com"`, the rule fails, and they
  are denied access:

```console
$ curl -H "user: alice@foo.com" localhost:5050/expenses/1
Not Authorized!
```

  If you aren’t seeing the same thing, make sure you created your policy
  correctly in `expenses.polar`.
{{% /callout %}}

## Using application data

We now have some basic access control in place, but we can do better.
Currently, anyone with an email ending in `@example.com` can see all expenses —
including expenses submitted by others.

{{% callout "Edit it!" "blue" %}}
  Let's modify our existing rule such that users can only see their own
  expenses:

  {{< literalInclude dynPath="expensesPath2"
                     fallback="expenses2" >}}
{{% /callout %}}

Behind the scenes, Oso looks up the `submitted_by` field on the provided
`Expense` instance and compares that value against the provided **actor**. And
just like that, an actor can only see an expense if they submitted it!

{{% callout "Try it!" "green" %}}
  Alice can see her own expenses but not Bhavik's:

```console
$ curl -H "user: alice@example.com" localhost:5050/expenses/1
Expense(...)
```

```console
$ curl -H "user: alice@example.com" localhost:5050/expenses/3
Not Authorized!
```

  ```console
  $ curl -H "user: alice@example.com" localhost:5050/expenses/3
  Not Authorized!
  ```
{{% /callout %}}

Feel free to play around with the current policy and experiment with adding
your own rules!

For example, if you have `Expense` and `User` classes defined in your
application, you could write a policy rule in Oso that says a `User` may
`"approve"` an `Expense` if they manage the `User` who submitted the expense
and the expense’s amount is less than $100.00:

{{< code file="expenses.polar" >}}
allow(approver: User, "approve", expense: Expense) if
    approver = expense.{{% exampleGet "submitted_by" %}}.{{% exampleGet "manager" %}}
    and expense.{{% exampleGet "amount" %}} < 10000;
{{< /code >}}

In the process of evaluating that rule, the Oso engine would call back into the
application in order to make determinations that rely on application data, such
as:

- Which user submitted the expense in question?
- Who is their manager?
- Is their manager the user who’s attempting to approve the expense?
- Does the expense’s `amount` field contain a value less than $100.00?

For more on leveraging application data in an Oso policy, check out
[Application Types](policies#application-types).

## Want to talk it through?

If you have any questions, are getting stuck, or just want to talk something
through, jump into [Slack](https://join-slack.osohq.com/) and an engineer from
the core team (or one of the hundreds of developers in the growing community)
will help you out.

{{% callout "What's next" "blue" %}}

- Explore how to [add Oso to an application](application).
- [Use Oso Roles](/guides/new-roles) to add Role-Based Access Control to your application
- Dive into [writing policies](policies) in detail.

{{% /callout %}}
