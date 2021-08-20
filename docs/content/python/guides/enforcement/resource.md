---
title: "Resource-level Enforcement"
weight: 10
any: true
# draft: true
---

{{% callout "Note: Preview API" %}}
  This is a preview API, meaning that it is not yet officially released. You may
  find other docs that conflict with the guidance here, so proceed at your own
  risk! If you have any questions, don't hesitate to [reach out to us on
  Slack](https://join-slack.osohq.com). We're here to help.
{{% /callout %}}

<div class="pb-10"></div>

# Resource-level Enforcement

Is the current user allowed to perform a certain action on a certain resource?
This is the central question of **"resource-level" enforcement.**

Resource-level enforcement is the bread-and-butter of application authorization.
If you only perform one type of authorization in your app, it should be
this. **Just about every endpoint in your application should perform some kind
of resource-level enforcement.**

The method you use for resource-level authorization is called `authorize`, and
is exposed by an Oso enforcer. The `authorize` method takes three arguments,
`user`, `action`, and `resource`. It doesn't return anything, but potentially
throws an error. To handle this error, see [Authorization
Failure](#authorization-failure).

<!-- You'll see this method in a lot of our guides and examples, because it's the
simplest way to use Oso in your app. -->

```python
oso.authorize(user, "approve", expense)
```

The `authorize` method checks all of the `allow` rules in your policy, and
ensures that there is an `allow` rule for that applies to the given user,
action, and resource:

```polar
allow(user: User, "approve", expense: Expense) if
    expense.org = org and
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
resource? In that case, the `authorize` method will raise an
`AuthorizationError`. There are actually two types of authorization errors,
depending on the situation.

- `NotFound` errors are for situations where the user should not even know
  that a particular resource _exists_. That is, the user does not have
  `"read"` permission on the resource. **You should handle these errors by
  showing the user a 404 "Not Found" error**.
- `Forbidden` errors are raised when the user knows that a resource exists
  (i.e. when they have permission to `"read"` the resource), but they are not
  allowed to perform the given action. **You should handle these errors by
  showing the user a 403 "Forbidden" error.**

{{% minicallout %}}
**Note**: a call to `authorize` with a `"read"` action will never raise a
`Forbidden` error, only `NotFound` errors—if the user is not allowed to read
the resource, the server should act as though it doesn't exist.
{{% /minicallout %}}

You could handle these errors at each place you call `authorize`, but that would
mean a lot of error handling. We recommend handling `NotFound` and `Forbidden`
errors globally in your application, using middleware or something similar.
Ideally, you can perform resource-level authorization by adding a single line of
code to each endpoint.

As an example, here's what a global error handler looks like in a flask app:

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
