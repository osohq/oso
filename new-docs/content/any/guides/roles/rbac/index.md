---
date: "2021-01-07T02:46:33.217Z"
docname: using/examples/rbac
images: {}
path: /using-examples-rbac
title: Add Basic Roles
description: |
  Learn how to add basic roles to your application, no matter what data store/ORM you use.
aliases:
  - ../../../using/examples/rbac.html
---

# Add Basic Roles to Your Application

Role-based access control (RBAC) refers to an authorization system that
groups permissions into roles that are assigned to actors, rather than
assigning permissions to actors directly. A permission is the ability to
perform an [action](glossary#actions) on a [resource](glossary#resources). In
an RBAC system, permissions are grouped into discrete roles (e.g., an "admin" role or
"manager" role), and these roles are assigned to actors to confer
permissions.

This guide shows an example of implementing basic [global
roles](learn/roles#global-roles) with Oso. Global Roles are roles that apply
to users and resources globally across the application.

For guidance on other RBAC use cases, check out [Role-Based Access Control Patterns](learn/roles).

## RBAC Basics

Representing roles in our policy is as simple as creating `role()`
[rules](polar-syntax#rules):

{{< literalInclude path="examples/rbac/01-simple.polar"
                   from="roles-start"
                   to="roles-end" >}}

In the above snippet of Polar, we create three `role()` rules and match on the
`actor`’s name to assign them the appropriate role. Let’s write some **allow**
rules that leverage our new roles:

{{< literalInclude path="examples/rbac/01-simple.polar"
                   from="allows-start"
                   to="allows-end" >}}

To test that the roles are working, we can write a few [inline
queries](polar-syntax#inline-queries-) in the same Polar file:

{{< literalInclude path="examples/rbac/01-simple.polar"
                   from="inline-queries-start"
                   to="inline-queries-end" >}}

Inline queries run when the file is loaded, and check that the query after the
`?=` succeeds.

We have a working RBAC system, but at this point it’s not quite as flexible as
we’d like. For example, Deirdre is in the Accounting department, but she’s
_also_ an employee and should be able to submit her own expenses. We could
define a second **allow** rule enabling accountants to `“submit”` expenses, but
it would be better to avoid that duplication and write our policy in a way that
accurately mirrors the role relationships of our business domain. Since
accountants are also employees, we can extend our `role(actor, “employee”)`
rule as follows:

{{< literalInclude path="examples/rbac/02-simple.polar"
                   from="accountant-inherits-from-employee-start"
                   to="accountant-inherits-from-employee-end" >}}

Administrators should be able to do anything that accountants and employees
can, and we can grant them those permissions through the same inheritance
structure:

{{< literalInclude path="examples/rbac/02-simple.polar"
                   from="admin-inherits-from-accountant-start"
                   to="admin-inherits-from-accountant-end" >}}

Now we can write a few more tests to ensure everything is hooked up correctly:

{{< literalInclude path="examples/rbac/02-simple.polar"
                   from="inline-queries-start"
                   to="inline-queries-end" >}}

## RBAC with Existing Roles

Our accounting firm’s authorization scheme is flexible, hierarchical, and —
let’s just go ahead and say it — beautiful. However, it’s entirely based on
data that lives in our policy. One of the distinguishing features of Oso is the
ability to [reach into existing domain models](getting-started/policies#application-types) to retrieve
context for an authorization decision.

Imagine we have a `user_roles` database table that contains mappings
between users and the roles they’ve been assigned.

Our {{% exampleGet "langName" %}} application has the following `User` model
that can look up its assigned roles from the database:

{{< literalInclude dynPath="userClassPath"
                   fallback="userClass" >}}

By registering the `User` class with Oso, we can begin leveraging it from
within our policy:

{{< literalInclude dynPath="registeredUserClassPath"
                   from="user-start"
                   to="user-end"
                   fallback="registeredUserClass" >}}

Our policy currently expects actors to be simple strings, but we can update
that by adding the `User` type specializer to our `role()` rules:

{{< literalInclude path="examples/rbac/05-external.polar"
                   from="roles-start"
                   to="roles-end" >}}

Our policy is a bit more verbose now, but don’t let that distract from the
momentous shift that just occurred: by adding a single decorator to our
application model, we’re now able to write rich policy over the model’s fields
and methods… and we aren’t finished yet!

We’re still mapping users to roles in the policy despite having access to the
existing mappings through the `User.role()` method. Let’s amend that:

{{< literalInclude path="examples/rbac/06-external.polar"
                   from="roles-start"
                   to="roles-end" >}}

There’s something really powerful happening in the above that bears
highlighting: Oso allowed us to not only create policies over existing
application data but, crucially, _to arrange that data in novel ways_,
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

{{% callout "What's next" "blue" %}}

{{< ifLang "python" >}}
- Learn how to use roles with
[SQLAlchemy](guides/roles/sqlalchemy_roles).
{{< /ifLang >}}
- Read more about advanced [Role-Based Access Control](learn/roles) with Oso.

{{% /callout %}}
