---
title: Quickstart (5 min)
description: |
  Ready to get started? See Oso in action, and walk through adding roles to an app.
weight: 1
---

# Oso Quickstart

Oso is an open-source, batteries-included library for authorizing actions in your app.
Out of the box, Oso lets you give your users roles and lets you specify permissions for those roles.
Roles can be as simple as User and Admin, or as complex as a management hierarchy.

![Diagram showing an application hierarchy with site admins, store owners, and customers](/getting-started/quickstart/images/app-hierarchy.png)

Oso isn’t restricted to roles, though — you can replace any authorization code in your app with an Oso policy.

Why use Oso?
- Authorization always starts out simple, but can be increasingly difficult to manage as your app grows.
- Authorization is security and should be as reliable as possible.
- If you’re not an authorization expert, Oso’s design will guide you to best practices.

Oso is a library — it runs alongside your app code and doesn’t make any calls over the network.
Your data doesn’t leave your server. Oso also doesn’t persist user data inside the library, so you stay in control of your data.

Here’s how data flows between your app and the Oso library:

[DIAGRAM]

## Install the Oso library

```bash
pip install --upgrade oso
# Or find the Oso package on <[http://pypi.python.org/pypi/oso/](http://pypi.python.org/pypi/oso/)>
```
## Add Oso to your app
To start, `import` Oso, create an Oso instance, and enable roles.
(Roles are a new thing in Oso, and they’re hidden behind a feature flag.)

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

That enforcement can happen in the request handler, at the database layer, or in middleware — here, we’ve chosen to enforce in the request handler.
Here’s a Flask route that displays a page if this user is allowed to read the associated page.

```python
from flask import Flask

app = Flask(__name__)
@app.route("/some/page/<pagenum>")
def page_show(pagenum):
   page = Page.pages[int(pagenum)]
   if oso.is_allowed(
       get_user(), # the user doing the request
       "read", # the action we want to do
       page): # the resource we want to do it to

       return f'<h1>A Page</h1><p>this is page {pagenum}</p>'
   else:
       return f'<h1>Sorry</h1><p>You are not allowed to see this page</p>'
```

Oso denies requests unless you explicitly tell it to accept that sort of request.
You can tell Oso what requests to accept by providing it with a file full of rules, which we call a policy.

## Write an authorization policy
Here is a typical policy, written in our declarative language, **Polar**.
It lets any actor with the role `user` read a page, but only lets actors with the role `admin` write to a page.

We can load our example policy from a file with the extension `.polar`.

```python
oso.load_file("example.polar")
```

Here’s the example.polar file:

```polar
actor_role(actor, role) if
   resources = Page.pages and
   r in resources and
   actions = r.has_roles(actor) and
   action in actions and
   role = { name: action, resource: r };

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

An _actor_role_ rule expresses the relationship between an actor and a role object of the form `{name: "the-role-name", resource: TheResourceObject}`.
The `is_allowed` call in your Python code will look up this rule, so this is required.

A _resource_ rule governs access to a specific resource in your app — for instance, a page, an object, a route, or a database entry.
Here, there are two possible actions that can be done to a Page: "read" and "write."
`users` can `read`, but not `write`, to a `Page`.
`admins` are allowed to `write` to a `Page`.
The line `implies: ["user"]` says that `admins` can also do anything `users` are allowed to do; that is, `read` a `Page`.

There’s much more you can do with Polar, including defining parent-child relationships — we’re just scratching the surface here.

## Calling back into your Python code

You can call properties and methods on your Python objects from Polar.
These will defer control back to your app.
Oso leaves the decision of how to store role assignments up to you — you might choose to store those role assignments in a database, in memory, or create them dynamically.
Our `actor_role` rule calls your Python method `has_roles` to get all the roles for our actor.

```polar
actor_role(actor, role) if
     resources = Page.pages and
     r in resources and
     actions = r.has_roles(actor) and
     action in actions and
     role = { name: action, resource: r };
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

```python
# install Oso
# put this code in a file named example.py
# run:
# python example.py
# browse http://127.0.0.1:5000/some/page/2

class Page:
   pages = []

   def __init__(self, pagenum):
       self.pagenum = pagenum

   # in a real application the returned list would
   # include all the roles available for this actor
   # for now every actor has the role "user"
   def has_roles(self, actor):
       return ["user"]

   def get_pages():
       return Page.pages

Page.pages = [Page(0), Page(1), Page(2)]

class User:
   def __init__(self, name):
       self.name = name

# Get the user -
def get_user():
   return User("someuser")

def get_page(pagenum):
   return Page.pages[pagenum]

from oso import Oso
oso = Oso()
oso.enable_roles()
oso.register_class(Page)
oso.register_class(User)

from flask import Flask

app = Flask(__name__)
@app.route("/some/page/<pagenum>")
def page_show(pagenum):
   page = Page.pages[int(pagenum)]
   if oso.is_allowed(
       get_user(), # the user doing the request
       "read", # the action we want to do
       page): # the resource we want to do it to

       return f'<h1>A Page</h1><p>this is page {pagenum}</p>'
   else:
       return f'<h1>Sorry</h1><p>You are not allowed to see this page</p>'

# we can load our policy from a file, or from a string, as here
oso.load_str("""

  actor_role(actor, role) if
       resources = Page.pages and
       r in resources and
       actions = r.has_roles(actor) and
       action in actions and
       role = { name: action, resource: r };

   resource(_type: Page, "page", actions, roles) if
       actions = ["read", "write"] and
       roles = {
           admin: {
               permissions: ["write"],
               implies: ["user"]
           },
           user: {
               permissions: ["read"]
           }
       };

   """)

app.run()
```

{{% callout "What's next" "blue" %}}

- Explore how to [add Oso to an application](application).
- Dive into [writing policies](policies) in detail.

{{% /callout %}}
