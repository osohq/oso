---
title: Enforce authorization
description: |
    Add authorization enforcement throughout your application using the
    authorize API to reject or accept requests that users make.
weight: 2
---

# Enforce authorization

In [Model your authorization logic](model) you defined an authorization
policy. In this guide, we will cover using Oso's enforcement API to
accept or reject requests based on the policy.

## The allow rule

At the end of [Model your authorization logic](model) we defined an `allow`
rule. The `allow` rule is used for resource-level enforcement. It
accepts three arguments: an `actor` (who is making the request), an
`action` (what the actor wants to do), and a `resource` (the object that the
actor wants to perform the action on).

Here's the `allow` rule you wrote:

```polar
allow(actor, action, resource) if
	has_permission(actor, action, resource);
```

This rule succeeds if the `actor` `has_permission` to perform `action` on
`resource`. `actor`, `action`, and `resource` are all variables. They
are provided when you query the policy.

## Querying the policy

The Oso library queries the policy with inputs from your application. To
construct a query, you give a rule name and a list of parameters. The
query returns a result for every rule that succeeds for the parameters
specified. If there are multiple rules that succeed, multiple results
will be returned. If no rules succeed, no results are returned from the
query.

To enforce authorization, we query for the `allow` rule with a specific `actor`,
`action`, and `resource`.

## `authorize`

The `authorize` method queries the `allow` rule. If the query doesn't have any
results (no rules succeed), it throws an authorization error. You should handle
this exception and [return an error response to the
user](guides/enforcement/resource#authorization-failure).

The `authorize` method should be called any time you want to check if a user can
perform an action—-like "read" or "delete"-—on a resource.

Here's an example of using `authorize` to check if a user can `"read"` a
repository.

{{< literalInclude
    dynPath="routePath"
    fallback="todo"
    hlFrom="docs: begin-authorize"
    hlTo="docs: end-authorize"
    from="docs: begin-show-route"
    to="docs: end-show-route"
    >}}

The user and repository parameters are {{% lang %}} objects. Oso knows about
fields & methods on {{% lang %}} objects, and their types, so you can access
this data directly from your policy. You used the roles property of the user
object in the [`has_role` implementation](model#give-your-users-roles).

## What's next

Add more `authorize` calls throughout your application, wherever you
read or write data on behalf of a user.

We only covered one type of enforcement in this guide: resource-level
enforcement. Oso can also enforce access to fields on an object, requests, or
queries from an external data source. See [How to: Enforce
authorization](guides/enforcement) for more.

This is all you need have Oso setup in your application and authorizing requests. Next, you may want to:

- [Write your own rules:](write-rules) Write rules that extend your policy to
  fit your application's requirements.
- [Filter collections of data:](filter-data) Apply authorization to large
  collections of data that cannot be loaded into memory.
- [Dive deeper on modeling authorization in the how to section:](/guides) Learn
  about other ways to model authorization with Oso, like organization roles and
  cross-resource roles or attribute based access control.
