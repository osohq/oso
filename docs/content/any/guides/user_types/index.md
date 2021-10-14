---
date: "2021-01-07T02:46:33.217Z"
docname: using/examples/user_types
images: {}
path: /using-examples-user-types
title: Multiple Actor Types
description: |
  Applications may have multiple types of users. Frequently, internal user
  accounts for support reps, operations teams, or testing. Oso policies can
  recognize different user types & apply different rules when necessary,
  avoiding the need for multiple authorization systems.
aliases:
  - ../../../using/examples/user_types.html
draft: True
---

# Multiple Actor Types

Recall that in Oso, [actors](glossary#actors) represent request-makers, the
“who” of an authorization request. Actors are commonly human users, but might
also be machines, servers, or other applications. Many applications support
multiple types of actors, and often different actor types require different
authorization logic.

In this guide, we’ll walk through a policy for an application with two actor
types: **Customers** and **Internal Users**.

{{% callout "Note" "blue" %}}
This guide assumes you are familiar with terms from Oso’s
[glossary](glossary).
{{% /callout %}}

## A Tale of Two Actors

Our example application has customers and internal users. Customers are allowed
to access the customer dashboard, and internal users are allowed to access the
customer dashboard as well as an internal dashboard. We can write a simple
policy to express this logic.

Let’s start by defining {{% exampleGet "langName" %}} classes to represent
customers and internal users:

{{< literalInclude dynPath="userClassesPath"
                   from="classes-start"
                   to="classes-end" >}}

We can now write a simple policy over these actor types:

{{< literalInclude dynPath="userPolicyPath"
                   from="simple-start"
                   to="simple-end" >}}

This policy uses [specialized
rules](guides/policies#registering-application-types) to control rules
execution based on the actor type that is passed into the authorization
request.

To finish securing our dashboards, we need to **enforce** our policy by adding
authorization requests to our application. Where and how authorization requests
are used is up to the application developer.

For our example, making a request might look like this:

{{< literalInclude dynPath="userClassesPath"
                   from="app-start"
                   to="app-end" >}}

Hooray, our customer and internal dashboards are now secure!

## Adding Actor Attributes

Since we saved so much time on authorization, we’ve decided to add another
dashboard to our application, an **accounts dashboard**. The accounts dashboard
should only be accessed by **account managers** (a type of internal user).
Since we’re experts at securing dashboards, we should be able to add this
authorization logic to our policy in no time. A simple way to solve this
problem is with RBAC.

We can add a `role()` method to our `InternalUser` class:

{{< literalInclude dynPath="userClassesPath2"
                   from="internal-start"
                   to="internal-end" >}}

Then add the following rule to our policy:

{{< literalInclude dynPath="userPolicyPath"
                   from="rbac-start"
                   to="rbac-end" >}}

This example shows a clear benefit of using different classes to represent
different actor types: the ability to add custom attributes. We can add
attributes specific to internal users, like roles, to the `InternalUser` class
without adding them to all application users.

We’ve been able to secure the accounts dashboard with a few lines of code, but
we’re not done yet!

Account managers are also allowed to access **account data**, but only for
accounts that they manage. In order to implement this logic, we need to know
the accounts of each account manager.

This is a compelling case for creating a new actor type for account managers
that has a method for retrieving a collection of managed accounts:

{{< literalInclude dynPath="userClassesPath2"
                   from="account-start"
                   to="account-end" >}}

Since account managers are also internal users, we’ve made the `AccountManager`
type extend `InternalUser`. This means that our rules that specialize on
`InternalUser` will still execute for account managers (see [Resources with
Inheritance](guides/inheritance)).

For the purposes of this example, we'll assume that `AccountData` is a resource
that has an `{{% exampleGet "accountId" %}}` attribute. Let’s add the following
lines to our policy:

{{< literalInclude dynPath="userPolicyPath"
                   from="manager-start"
                   to="manager-end" >}}

The first rule replaces the RBAC rule we previously used to control access to
the accounts dashboard. The second rule controls access to account data.

We can update our application code slightly to generate `AccountManager` users:

{{< literalInclude dynPath="userClassesPath2"
                   from="generate-start"
                   to="generate-end" >}}

We’ve now successfully secured all three dashboards and customer account data.

## Summary

It is common to require different authorization logic for different types of
application users. In this example, we showed how to use different actor types
to represent different users in Oso. We wrote policies with rules that
specialized on the type of actor and even added attributes to some actor types
that we used in the policy. We also demonstrated how inheritance can be used to
match rules to multiple types of actors.
