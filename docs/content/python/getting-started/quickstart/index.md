---
title: Quickstart (5 min)
description: |
  Ready to get started? See Oso in action, and walk through adding roles to an app.
weight: 1
---

# Oso Quickstart

Oso is an open-source, batteries-included library for authorizing actions in your app.
Out of the box, Oso lets you give your users roles and lets you specify permissions for those roles.
Roles can be as simple as "user" and "admin", or as complex as a management hierarchy.

![Diagram showing an application hierarchy with site admins, store owners, and customers](/getting-started/quickstart/images/app-hierarchy.png)

Oso isn't restricted to roles, though — you can replace any authorization code in your app with an Oso policy.

Why use Oso? Authorization always starts out simple, but can be increasingly
difficult to manage as your app grows. Oso's design will guide you to best practices.

Oso is a library — it runs alongside your app code and doesn't make any calls over the network.
Your data doesn't leave your server. Oso also doesn't persist user data inside the library, so you stay in control of your data.

Here's how data flows between your app and the Oso library:

![Architecture diagram for Oso library loading a polcicy file and making authorization decisions. ](/getting-started/quickstart/images/arch-simple.png)
## Install the Oso library

```bash
pip install --upgrade oso
# Or find the Oso package on <[http://pypi.python.org/pypi/oso/](http://pypi.python.org/pypi/oso/)>
```
## Add Oso to your app
To start, `import` Oso, create an Oso instance, and enable roles.
Enabling roles gives you access to Oso's builtin support for
role-based access control. 

```python
from oso import Oso
oso = Oso()
oso.enable_roles()
```

## Accept or deny requests

When a request arrives, your application will ask Oso if it should accept the request. Oso needs three pieces of information to make that decision:
- Who is making the request? (the "actor")
- What are they trying to do? (the "action")
- What are they doing it to? (the "resource")

In Oso, these pieces of information are passed to the `is_allowed` call: `is_allowed(actor, action, resource)`.
`is_allowed` will return `True` or `False`, and your application can choose how to enforce that decision.

That enforcement can happen in the request handler, at the database layer, or in middleware — here, we've chosen to enforce in the request handler.
Here's a Flask route that displays a page if this user is allowed to read the associated page.

```python
from flask import Flask

app = Flask(__name__)
@app.route("/page/<pagenum>")
def page_show(pagenum):
    page = Page.get_page(pagenum)
    if oso.is_allowed(
        User.get_current_user(),  # the user doing the request
        "read",  # the action we want to do
        page,  # the resource we want to do it to
    ):
        return f"<h1>A Page</h1><p>this is page {pagenum}</p>", 200
    else:
        return f"<h1>Sorry</h1><p>You are not allowed to see this page</p>", 403
```

Oso denies requests unless you explicitly tell it to accept that sort of request.
You can tell Oso what requests to accept by providing it with a file full of rules, which we call a policy.

## Write an authorization policy
Here is a typical policy, written in our declarative language, **Polar**.
It lets any actor with the role `"user"` read a page, but only actors with the role `"admin"` can write to a page.

We can load our example policy from a file with the extension `.polar`.

```python
oso.load_file("authorization.polar")
```

Here's the authorization.polar file:

```polar
allow(actor, action, resource) if
    role_allow(actor, action, resource);

actor_role(actor, role) if
    role in actor.get_roles();

resource(_type: Page, "page", actions, roles) if
    actions = ["read", "write"] and
    roles = {
        user: {
            permissions: ["read"]
        },
        admin: {
            permissions: ["write"],
            implies: ["user"]
        }
    };
 ```

The `allow` rule is the top-level rule that we use to say who can do what in our application.
In this case, we are delegating to Oso's builtin `role_allow` rule which implements all the
authorization logic for role-based access control based on the data we provide in `actor_role`
and `resource`.

An `actor_role` rule expresses the relationship between an actor and a role object of the form `{name: "the-role-name", resource: TheResourceObject}`.
The `is_allowed` call in your Python code will look up this rule, so this is required.

A _resource_ rule governs access to a specific resource in your app — for instance, a page, an object, a route, or a database entry.
Here, there are two possible actions that can be done to a `Page`: `"read"` and `"write"`.
Users can read, but not write, to a `Page`.
Admins are allowed to write to a `Page`.

The line `implies: ["user"]` says that admins can also do anything users are allowed to do; that is, read a `Page`.

There's much more you can do with Polar, including defining parent-child relationships — we're just scratching the surface here.

## Calling back into your Python code

You can call properties and methods on your Python objects from Polar.
These will defer control back to your app.
Oso leaves the decision of how to store role assignments up to you — you might choose to store those role assignments in a database, in memory, or create them dynamically.
Our `actor_role` rule calls the Python method `has_roles` to get all the roles for our actor.

```polar
actor_role(actor, role) if
    role in actor.get_roles();
 ```

To refer to your Python classes in Polar, you must `register` them with Oso.

```python
from oso import Oso
oso = Oso()

oso.register_class(Page)
oso.register_class(User)
```

## Want to talk it through?

If you have any questions, are getting stuck, or just want to talk something
through, jump into [Slack](https://join-slack.osohq.com/) and an engineer from
the core team (or one of the hundreds of developers in the growing community)
will help you out.

## Complete Running Example

Download and run the code locally by cloning the {{% exampleGet "githubApp" %}}:

```console
git clone {{% exampleGet "githubURL" %}}
```

{{% callout "What's next" "blue" %}}

- Explore how to [add Oso to an application](application).
- Dive into [writing policies](policies) in detail.

{{% /callout %}}
