---
date: "2021-01-07T02:46:33.217Z"
docname: using/examples/context
images: {}
path: /using-examples-context
title: Use Context in Policies
weight: 5
description: |
  Pass context beyond the actor, action, and resource into Oso policies.
aliases:
  - ../../../using/examples/context.html
---

# Using Additional Context in Policies

Allow rules take in an [actor](glossary#actors) (which comes from authorization
logic) and a [resource](glossary#resources) (which comes from mapping).
Sometimes you need some additional context information about the environment to
write rules over.

For example, let’s say you have a policy like this:

{{< literalInclude path="examples/context/01-context.polar"
                   from="admin-start"
                   to="admin-end" >}}

Here we have a very simple allow rule that allows an actor to access any
resource if they are an admin. Maybe we want to also let any actor access any
resource when the app is in development mode. A typical way to flag that an
app is running in development or production mode would be to set an environment
variable, e.g. `ENV=development` or `ENV=production`.

How would we read that environment variable from our policy though?

We can use a application class that lets us directly access the environment
variables:

{{< literalInclude dynPath="envClassPath"
                   from="context-start"
                   to="context-end" >}}

The above class exposes a `var` method that reads the application’s environment
variables and returns the requested value. We register the class with Oso,
allowing us to instantiate it in the policy.

We can add a new `allow` rule that permits an actor to access a resource if the
application is in development mode:

{{< literalInclude path="examples/context/01-context.polar"
                   from="env-start"
                   to="env-end" >}}

## Summary

Application classes make it easy to expose any sort of application data to your
policy, including environment variables and request context. This simple
pattern lets you expose any kind of data you want to use in your policy, not
just `Actor` and `Resource` classes.
