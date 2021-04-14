---
date: '2021-01-07T02:46:33.217Z'
docname: using/examples/abac
images: {}
path: /using-examples-abac
title: Attribute-Based Access Control
description: |
    Attribute-based access control relies on rich attributes associated with
    each actor to make authorization decisions. This model is often used when
    RBAC is not expressive enough, and is a natural extension of RBAC when
    using Oso.
aliases:
    - ../../../using/examples/abac.html
weight: 2
---

# Attribute-Based Access Control (ABAC)

Whereas RBAC allows you to group users and permissions into predefined buckets,
you may also want to represent fine-grained or dynamic permissions based on
*who* the user is and her relation to the resource she wants to access. This is
known as [attribute-based access
control](https://en.wikipedia.org/wiki/Attribute-based_access_control) (ABAC).

## ABAC Basics

Suppose we want to allow employees to view *their own* expenses.

We can register our user class with Oso:

{{< literalInclude dynPath="userClassPath"
                   from="user-class-start"
                   to="user-class-end"
                   fallback="userClass" >}}

We can do the same with the resource being requested:

{{< literalInclude dynPath="expenseClassPath"
                   from="expense-class-start"
                   to="expense-class-end"
                   fallback="expenseClass" >}}

An `allow` rule that checks that the user reading the
expense is the same person who submitted the expense, would look like:

{{< literalInclude path="examples/abac/01-simple.polar"
                   from="rule-start"
                   to="rule-end" >}}

This simple example shows the potential for ABAC: we took an intuitive concept
of “can see their own expenses” and represented it as a single comparison.

The power of ABAC comes from being able to express these kind of permissions
based on who you are and how you are related to the data.

## ABAC ❤️ RBAC

As alluded to in the summary on RBAC, provisioning access based on checking
whether a user has a particular role is technically a simple variant of ABAC.
Putting aside whether this is a relevant distinction, the two are closely
related.

The power of RBAC comes from: adding some form of organization to the limitless
distinct permissions that a person might have, and exposing those in an
intuitive, human-understandable way.

Combine this with what ABAC does best: representing relations between a user
and the data, and you get intuitive, but fine-grained permissions. For example,
suppose our company has taken off and now spans multiple locations, and now
accountants can only view expenses from their own locations. We can combine our
previous roles with some simple ABAC conditions to achieve this:

{{< literalInclude path="examples/abac/02-rbac.polar"
                   from="simple-rule-start"
                   to="simple-rule-end" >}}

This is great when what we need is an intersection of models, and you want to
apply both RBAC and ABAC policies simultaneously. However, the ABAC model
can be even more powerful when composed with roles. And having the roles themselves
include attributes.

For example, an employee might be an administrator of a *project*,
and therefore is allowed to see all expenses related to that project.

{{< literalInclude path="examples/abac/02-rbac.polar"
                   from="project-rule-start"
                   to="project-rule-end" >}}

What we can see is happening here, is that we are associated roles not just
globally to a user, but to a user for some specific resource. Other examples
might be team-, or organization- specific roles.

And these can also follow inheritance patterns like we saw with regular roles.

{{< literalInclude path="examples/abac/02-rbac.polar"
                   from="role-inherit-start"
                   to="role-inherit-end" >}}

## Hierarchies

Up to this point, we’ve made a big deal about ABAC being able to represent
relations between users and resources. In the previous example, we even showed
how relations between resources permits creating inheritance logic. To expand
on that idea, here we look at representing organizational hierarchies in our
policy.

Starting out with a simple example, suppose managers can view employees’
expenses:

{{< literalInclude path="examples/abac/03-hierarchy.polar"
                   from="start-simple-rule"
                   to="end-simple-rule" >}}

First thing we can do, is extract out the logic for checking whether the user
manages someone:

{{< literalInclude path="examples/abac/03-hierarchy.polar"
                   from="start-manages-rule"
                   to="end-manages-rule" >}}

Now if we want this logic to apply for managers, and managers’ managers, and so
on… then we need to make sure this logic is evaluated recursively:

{{< literalInclude path="examples/abac/03-hierarchy.polar"
                   from="start-hierarchy-rule"
                   to="end-hierarchy-rule" >}}

<!-- TODO: Summary -->
