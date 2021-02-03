---
date: '2021-01-07T02:46:33.217Z'
docname: using/examples/abac
images: {}
path: /using-examples-abac
title: ABAC
description: |
    Attribute-based access control relies on rich attributes associated with
    each actor to make authorization decisions. This model is often used when
    RBAC is not expressive enough, and is a natural extension of RBAC when
    using oso.
aliases: 
    - ../../../using/examples/abac.html
---

# ABAC

Whereas RBAC allows you to group users and permissions into predefined buckets,
you may also want to represent fine-grained or dynamic permissions based on
*who* the user is and her relation to the resource she wants to access. This is
known as [attribute-based access
control](https://en.wikipedia.org/wiki/Attribute-based_access_control) (ABAC).

## ABAC Basics

Suppose we want to allow employees to view *their own* expenses.

We can register our user classes with oso:

{{% exampleGet "userClass" %}}

We can do the same with the resources being requested:

{{% exampleGet "expenseClass" %}}

An `allow` rule that checks that the user reading the
expense is the same person who submitted the expense, would look like:

```polar
allow(actor: User, "view", resource: Expense) if
    resource.{{% exampleGet "submitted_by" %}} = actor.name;
```

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

```polar
# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) if
    role(actor, "accountant") and
    actor.location = resource.location;
```

This is great when what we need is an intersection of models, and you want to
apply both RBAC and ABAC policies simultaneously. However, the ABAC model
can be even more powerful when composed with roles. And having the roles themselves
include attributes.

For example, an employee might be an administrator of a *project*,
and therefore is allowed to see all expenses related to that project.

```polar
# Alice is an admin of Project 1
role(_: User { name: "alice" }, "admin", _: Project { id: 1 });

# Project admins can view expenses of the project
allow(actor: User, "view", resource: Expense) if
    role(actor, "admin", Project.id(resource.project{{% exampleGet "postfixId" %}}));
```

What we can see is happening here, is that we are associated roles not just
globally to a user, but to a user for some specific resource. Other examples
might be team-, or organization- specific roles.

And these can also follow inheritance patterns like we saw with regular roles.

```polar
# Bhavik is an admin of ACME
role(_: User { name: "bhavik" }, "admin",  _: Organization { name: "ACME" });

# Team roles inherit from Organization roles
role(actor: User, role: String, team: Team) if
    role(actor, role, Organization.id(team.organization{{% exampleGet "postfixId" %}}));

# Project roles inherit from Team roles
role(actor: User, role: String, project: Project) if
    role(actor, role, Team.id(project.team{{% exampleGet "postfixId" %}}));
```

## Hierarchies

Up to this point, we’ve made a big deal about ABAC being able to represent
relations between users and resources. In the previous example, we even showed
how relations between resources permits creating inheritance logic. To expand
on that idea, here we look at representing organizational hierarchies in our
policy.

Starting out with a simple example, suppose managers can view employees’
expenses:

```polar
allow(actor: User, "view", resource: Expense) if
    employee in actor.employees() and
    employee.name = resource.{{% exampleGet "submitted_by" %}};
```

First thing we can do, is extract out the logic for checking whether the user
manages someone:

```polar
allow(actor: User, "view", resource: Expense) if
    manages(actor, employee) and
    employee.name = resource.{{% exampleGet "submitted_by" %}};
```

Now if we want this logic to apply for managers, and managers’ managers, and so
on… then we need to make sure this logic is evaluated recursively:

```polar
# Management hierarchies
manages(manager: User, employee) if
    report in manager.employees()
    and (report = employee or manages(report, employee));
```

<!-- TODO: Summary -->
