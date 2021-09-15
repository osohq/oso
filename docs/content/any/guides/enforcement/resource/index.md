---
title: "Resource-level Enforcement"
weight: 10
any: true
description: >
  Learn about enforcing request-level authorization, controlling who can perform
  which actions on which resources.
---

# Resource-level Enforcement

Is the current user allowed to perform a certain action on a certain resource?
This is the central question of **"resource-level" enforcement.**

- Can a user update settings for this organization?
- Can a user read this repository?
- Can an admin resend a password reset email for this user?

Resource-level enforcement is the bread-and-butter of application authorization.
If you only perform one type of authorization in your app, it should be
this. **Just about every endpoint in your application should perform some kind
of resource-level enforcement.**

## Authorize an action

The method to use for resource-level authorization is called {{% exampleGet "authorizeLink" %}}. You use this method to ensure that
a user has permission to perform a particular _action_ on a particular _resource._
The `{{< exampleGet "authorize" >}}` method takes three arguments, `user`, `action`, and `resource`.
It doesn't return anything when the action is allowed, but throws an error when
it is not. To handle this error, see [Authorization
Failure](#authorization-failure).

<!-- You'll see this method in a lot of our guides and examples, because it's the
simplest way to use Oso in your app. -->

{{% exampleGet "exampleCall" %}}

The `{{< exampleGet "authorize" >}}` method checks all of the `allow` rules in your policy and
ensures that there is an `allow` rule that applies to the given user,
action, and resource, like this:

```polar
allow(user: User, "approve", _expense: Expense) if
    user.{{< exampleGet "isAdmin" >}};
```

Let's see an example of `{{< exampleGet "authorize" >}}` from within an endpoint:

{{% exampleGet "approveExpense" %}}

As you can see from this example, it's common to have to fetch _some_ data
before performing authorization. To perform resource-level authorization, you
normally need to have the resource loaded!

## Authorization Failure

What happens when the authorization fails? That is, what if there is not an
`allow` rule that gives the user permission to perform the action on the
resource? In that case, the `{{< exampleGet "authorize" >}}` method will raise
{{% exampleGet "authorizationErrorLink" %}}. There are actually two types of authorization
errors, depending on the situation.

- {{% exampleGet "notFoundErrorLink" %}} errors are
  for situations where the user should not even know that a particular resource
  _exists_. That is, the user does not have `"read"` permission on the resource.
  **You should handle these errors by showing the user a 404 "Not Found"
  error**.
- {{% exampleGet "forbiddenErrorLink" %}} errors are
  raised when the user knows that a resource exists (i.e. when they have
  permission to `"read"` the resource), but they are not allowed to perform the
  given action. **You should handle these errors by showing the user a 403
  "Forbidden" error.**

{{% minicallout %}}
**Note**: a call to `{{< exampleGet "authorize" >}}` with a `"read"` action will never raise a
`{{< exampleGet "forbiddenError" >}}` error, only `{{< exampleGet "notFoundError" >}}` errors—if the user is not allowed to read
the resource, the server should act as though it doesn't exist.
{{% /minicallout %}}

You could handle these errors at each place you call `{{< exampleGet "authorize" >}}`, but that would
mean a lot of error handling. We recommend handling `{{< exampleGet "notFoundError" >}}` and `{{< exampleGet "forbiddenError" >}}`
errors globally in your application, using middleware or something similar.
Ideally, you can perform resource-level authorization by adding a single line of
code to each endpoint.

{{% exampleGet "globalErrorHandler" %}}
