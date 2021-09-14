---
title: "Resource-level Enforcement"
weight: 10
any: true
description: >
  Learn about enforcing request-level authorization, controlling who can perform
  which actions on which resources.

# draft: true
---

{{% callout "Note: 0.20.0 Beta Feature" %}}
  This is an API provided by the beta release of Oso 0.20.0, meaning that it is
  not yet officially released. You may find other docs that conflict with the
  guidance here, so proceed at your own risk! If you have any questions, don't
  hesitate to [reach out to us on Slack](https://join-slack.osohq.com). We're
  here to help.
{{% /callout %}}

<div class="pb-10"></div>

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

## Use the `authorize` method

The method to use for resource-level authorization is called {{% apiDeepLink
class="Oso" %}}authorize{{% /apiDeepLink %}}. You use this method to ensure that
a user has permission to perform a particular _action_ on a particular _resource._
The `authorize` method takes three arguments, `user`, `action`, and `resource`.
It doesn't return anything when the action is allowed, but throws an error when
it is not. To handle this error, see [Authorization
Failure](#authorization-failure).

<!-- You'll see this method in a lot of our guides and examples, because it's the
simplest way to use Oso in your app. -->

```python
oso.authorize(user, "approve", expense)
```

The `authorize` method checks all of the `allow` rules in your policy, and
ensures that there is an `allow` rule for that applies to the given user,
action, and resource, like this:

```polar
allow(user: User, "approve", expense: Expense) if
    org = expense.org and
    user.has_org_role(org, "admin");
```

Let's see an example of `authorize` from within an endpoint:

```python
def get_expense(user, expense_id):
    expense = db.fetch(
        "SELECT * FROM expenses WHERE id = %", expense_id)
    oso.authorize(user, "read", expense)

    # ... process request
```

As you can see from this example, it's common to have to fetch _some_ data
before performing authorization. To perform resource-level authorization, you
normally need to have the resource loaded!

## Authorization Failure

What happens when the authorization fails? That is, what if there is not an
`allow` rule that gives the user permission to perform the action on the
resource? In that case, the {{% apiDeepLink class="Oso" %}}authorize{{%
/apiDeepLink %}} method will raise an {{% apiDeepLink module="oso.exceptions"
class="AuthorizationError" /%}}. There are actually two types of authorization
errors, depending on the situation.

- {{< apiDeepLink module="oso.exceptions" class="NotFoundError" />}} errors are
  for situations where the user should not even know that a particular resource
  _exists_. That is, the user does not have `"read"` permission on the resource.
  **You should handle these errors by showing the user a 404 "Not Found"
  error**.
- {{< apiDeepLink module="oso.exceptions" class="ForbiddenError" />}} errors are
  raised when the user knows that a resource exists (i.e. when they have
  permission to `"read"` the resource), but they are not allowed to perform the
  given action. **You should handle these errors by showing the user a 403
  "Forbidden" error.**

{{% minicallout %}}
**Note**: a call to `authorize` with a `"read"` action will never raise a
`ForbiddenError` error, only `NotFoundError` errors—if the user is not allowed to read
the resource, the server should act as though it doesn't exist.
{{% /minicallout %}}

You could handle these errors at each place you call `authorize`, but that would
mean a lot of error handling. We recommend handling `NotFoundError` and `ForbiddenError`
errors globally in your application, using middleware or something similar.
Ideally, you can perform resource-level authorization by adding a single line of
code to each endpoint.

As an example, here's what a global error handler looks like in a Flask app:

```python
from oso import ForbiddenError, NotFoundError

app = Flask()

@app.errorhandler(ForbiddenError)
def handle_forbidden(*_):
    return {"message": "Forbidden"}, 403

@app.errorhandler(NotFoundError)
def handle_not_found(*_):
    return {"message": "Not Found"}, 404
```

Then, when your application calls `oso.authorize(user, action, resource)`, it
will know how to handle errors that arise.
