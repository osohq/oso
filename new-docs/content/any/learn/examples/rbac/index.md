---
date: '2021-01-07T02:46:33.217Z'
docname: using/examples/rbac
images: {}
path: /using-examples-rbac
title: Global RBAC
description: |
    Role-based access control (RBAC) assigns each actor a role. Instead of
    granting permissions to individual actors, they are granted to roles.
aliases: 
    - ../../../using/examples/rbac.html
---

# Global RBAC

Many authorization systems in the wild are built on a [role-based access
control](https://en.wikipedia.org/wiki/Role-based_access_control) model. The
general thesis of RBAC is that the set of permissions for a system — a
permission being the ability to perform an [action](glossary#actions) on a
[resource](glossary#resources) — can be grouped into roles.

This guide shows an example of implementing [global
roles](learn/roles/patterns#global-roles). For guidance on other RBAC use
cases, check out [Role-Based Access Control Patterns](learn/roles/patterns).

## RBAC Basics

Representing roles in our policy is as simple as creating `role()`
[rules](polar-syntax#rules):

```polar
role(actor: String, "employee") if
    actor = "alice" or
    actor = "bhavik" or
    actor = "cora";

role(actor: String, "accountant") if
    actor = "deirdre" or
    actor = "ebrahim" or
    actor = "frantz";

role(actor: String, "admin") if
    actor = "greta" or
    actor = "han" or
    actor = "iqbal";
```

In the above snippet of Polar, we create three `role()` rules and match on the
`actor`’s name to assign them the appropriate role. Let’s write some **allow**
rules that leverage our new roles:

```polar
# Employees can submit expenses
allow(actor: String, "submit", "expense") if
    role(actor, "employee");

# Accountants can view expenses
allow(actor: String, "view", "expense") if
    role(actor, "accountant");

# Admins can approve expenses
allow(actor: String, "approve", "expense") if
    role(actor, "admin");
```

To test that the roles are working, we can write a few [inline
queries](polar-syntax#inline-queries-) in the same Polar file:

```polar
# Deirdre the accountant can view expenses
?= allow("deirdre", "view", "expense");

# but cannot submit or approve them
?= not allow("deirdre", "submit", "expense");
?= not allow("deirdre", "approve", "expense");
```

Inline queries run when the file is loaded, and check that the query after the
`?=` succeeds.

We have a working RBAC system, but at this point it’s not quite as flexible as
we’d like. For example, Deirdre is in the Accounting department, but she’s
*also* an employee and should be able to submit her own expenses. We could
define a second **allow** rule enabling accountants to `“submit”` expenses, but
it would be better to avoid that duplication and write our policy in a way that
accurately mirrors the role relationships of our business domain. Since
accountants are also employees, we can extend our `role(actor, “employee”)`
rule as follows:

```polar
# Accountants can do anything an employee can do
role(actor, "employee") if
    actor = "alice" or
    actor = "bhavik" or
    actor = "cora" or
    role(actor, "accountant");
```

Administrators should be able to do anything that accountants and employees
can, and we can grant them those permissions through the same inheritance
structure:

```polar
# Admins can do anything an accountant can do
role(actor, "accountant") if
    actor = "deirdre" or
    actor = "ebrahim" or
    actor = "frantz" or
    role(actor, "admin");
```

Now we can write a few more tests to ensure everything is hooked up correctly:

```polar
# Deirdre the accountant can view and submit expenses
?= allow("deirdre", "view", "expense");
?= allow("deirdre", "submit", "expense");

# but cannot approve them
?= not allow("deirdre", "approve", "expense");

# Iqbal the administrator can do everything
?= allow("iqbal", "view", "expense");
?= allow("iqbal", "submit", "expense");
?= allow("iqbal", "approve", "expense");
```

## RBAC with Existing Roles

Our accounting firm’s authorization scheme is flexible, hierarchical, and —
let’s just go ahead and say it — beautiful. However, it’s entirely based on
data that lives in our policy. One of the distinguishing features of Oso is the
ability to [reach into existing domain models](application-types) to retrieve
context for an authorization decision.

Imagine we have a `user_roles` database table that contains mappings
between users and the roles they’ve been assigned.

Our {{% exampleGet "langName" %}} application has the following `User` model
that can look up its assigned roles from the database:

{{% exampleGet "userClass" %}}

By registering the `User` class with Oso, we can begin leveraging it from
within our policy. Our policy currently expects actors to be simple strings,
but we can update that by adding the `User` type specializer to our `role()`
rules:

```polar
role(actor: User, "employee") if
    actor.name = "alice" or
    actor.name = "bhavik" or
    actor.name = "cora" or
    role(actor, "accountant");

role(actor: User, "accountant") if
    actor.name = "deirdre" or
    actor.name = "ebrahim" or
    actor.name = "frantz" or
    role(actor, "admin");

role(actor: User, "admin") if
    actor.name = "greta" or
    actor.name = "han" or
    actor.name = "iqbal";
```

Our policy is a bit more verbose now, but don’t let that distract from the
momentous shift that just occurred: by adding a single decorator to our
application model, we’re now able to write rich policy over the model’s fields
and methods… and we aren’t finished yet!

We’re still mapping users to roles in the policy despite having access to the
existing mappings through the `User.role()` method. Let’s amend that:

```polar
role(actor: User, "employee") if
    actor.role = "employee" or
    role(actor, "accountant");

role(actor: User, "accountant") if
    actor.role = "accountant" or
    role(actor, "admin");

role(actor: User, "admin") if
    actor.role = "admin";
```

There’s something really powerful happening in the above that bears
highlighting: Oso allowed us to not only create policies over existing
application data but, crucially, *to arrange that data in novel ways*,
enriching the pool of contextual data that informs authorization decisions
without littering complex logic all over the application. The hierarchy we
created among the `“admin”`, `“accountant”`, and `“employee”` roles extends the
existing authorization data but lives entirely in the authorization policy and
required **zero** new application code.

## Summary

We started with the basics of RBAC by writing out a toy policy and assigning
roles to actors in Polar. We saw how simple it is to construct arbitrary role
hierarchies, and we added a few inline queries to test our policy.

Things started to get really interesting when we registered the `User` model
with Oso, with that one-line change in our application code unlocking the
powerful pattern of writing authorization logic directly over the fields and
methods of our existing application model.

We were able to use one of those existing methods, `User.role()`, to write
rules over the role data stored in our application’s relational database. But
we took it a step further and rearranged the existing application roles
(`“admin”`, `“accountant”`, and `“employee”`) into a hierarchy that extended
the application’s authorization system without requiring any changes to core
application code.

The seasoned vets in the audience may have recognized the `actor.role`
attribute lookup for what it is: a pinch of [attribute-based access
control](https://en.wikipedia.org/wiki/Attribute-based_access_control) (ABAC)
hiding amongst our RBAC policy. In the next section, we’ll dive fully into
attribute-based authorization and show how intuitive it is to write concise,
flexible, and powerful ABAC rules with Oso.
