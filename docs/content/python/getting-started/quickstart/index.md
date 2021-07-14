---
title: Quickstart (5 min)
description: |
  Ready to get started? See Oso in action, and walk through our quick
  tutorial for adding authorization to a simple web server.
weight: 1
---

<!--

This guide is not setup to use literalInclude. As a result the
examples are manually maintained to match the quickstart repository.

This needs to be updated.

-->

# Quickstart

Oso is an open-source, batteries-included library for authorizing actions in your app.
Out of the box, Oso lets you give your users roles and lets you specify permissions for those roles.
Roles can be as simple as "guest" and "admin", or as complex as a management hierarchy.

Oso isn't restricted to roles, though — you can replace any authorization code in your app with an Oso policy.

Why use Oso?
Authorization always starts out simple, but it can be increasingly difficult to manage as your app grows.
Oso's design will guide you to best practices.

Oso is a library — it runs alongside your app code and doesn't make any calls over the network.
Your data doesn't leave your server. Oso also doesn't persist user data inside the library, so you stay in control of your data.

## Install the Oso library

{{% exampleGet "installation_new" %}}

## Add Oso to your app

To start, `{{% exampleGet "import" %}}` Oso, create an Oso instance, and enable roles.
Enabling roles gives you access to Oso's builtin support for
role-based access control. 

{{% exampleGet "import_code" %}}

## Accept or deny requests

When a request arrives, your application will need to ask Oso if it should accept the request. Oso needs three pieces of information to make that decision:
- Who is making the request? (the "actor")
- What are they trying to do? (the "action")
- What are they doing it to? (the "resource")

You'll pass these pieces of information are to Oso's `{{% exampleGet "isAllowed" %}}` method: `{{% exampleGet "isAllowed" %}}(actor, action, resource)`.
`{{% exampleGet "isAllowed" %}}` will return `True` or `False`, and your application can choose how to enforce that decision.

That enforcement can happen in the request handler, at the database layer, or in middleware — here, we've chosen to enforce in the request handler.
Here's {{% exampleGet "example_app" %}} route that only displays a page if the current user is allowed to read it.

{{% exampleGet "app_code" %}}


Oso denies requests unless you explicitly tell it to accept that sort of request.
You can tell Oso what requests to accept by providing it with a file full of rules, which we call a policy.

## Write an authorization policy
Here is a typical policy, written in our declarative language, **Polar**.
It lets any actor with the role `guest` read a page, but only actors with the role `admin` can write to a page.

We can load our example policy from a file with the extension `.polar`.

{{% exampleGet "load_policy" %}}

Here's the `authorization.polar` file:

```polar
allow(actor, action, resource) if
    role_allows(actor, action, resource);

actor_has_role_for_resource(actor, role_name, resource) if
    role in actor.get_roles() and
    role_name = role.name and
    resource = role.resource;

resource(_type: Page, "page", actions, roles) if
    actions = ["read", "write"] and
    roles = {
        user: {
            permissions: ["read"]
        },
        admin: {
            permissions: ["write"],
            implies: ["guest"]
        }
    };
 ```

The `allow` rule is the top-level rule that we use to say who can do what in our application.
In this case, we are delegating to Oso's built-in `role_allows` rule which implements all the
authorization logic for role-based access control based on the data we provide in `actor_has_role_for_resource`
and `resource`.

An `actor_has_role_for_resource` rule looks up role objects that are associated with an actor.
Role objects are of the form `{name: "the-role-name", resource: TheResourceObject}`.
The Oso builtin roles will look up this rule, so this is required.

A `resource` rule governs access to a specific resource in your app — for instance, a page, an object, a route, or a database entry.
In the above configuration for `Page`, there are two possible actions: `"read"` and `"write"`.
Guests can read, but not write, to a `Page`.
Admins are allowed to write to a `Page`.

The line `implies: ["guest"]` says that admins can also do anything guests are allowed to do; that is, admins can also read a `Page`.

There's much more you can do with Oso — we're just scratching the surface here. Polar is a very expressive language, and you can encode authorization logic far beyond what we've shown in this guide.

## Calling back into your {{% lang %}} code

You can call properties and methods on your {{% exampleGet "objects" %}} from Polar.
These will defer control back to your app.
Oso leaves the decision of how to store role assignments up to you — you might choose to store those role assignments in a database, in memory, or create them dynamically.
Our `actor_has_role_for_resource` rule calls the {{% exampleGet "methods" %}} `{{% exampleGet "getroles" %}}` to get all the roles for our actor.

```polar
actor_has_role_for_resource(actor, role_name, resource) if
    role in actor.get_roles() and
    role_name = role.name and
    resource = role.resource;
 ```

To refer to your {{% exampleGet "classes" %}} in Polar, you must _register_ them with Oso.

{{% exampleGet "register_classes" %}}

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
