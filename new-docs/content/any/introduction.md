---
title: Introduction
any: true
hideContents: true
---

### What is oso?

oso is an **open source library for authorization**. You use it to define what users
can and cannot do in your application.

Think about oso as providing you a structured way to implement concepts like
"users can see their own data", or to fast-track adding common access control
patterns like role-based access control.

{{< callout "Ready to go?" "primary" >}}
Dive straight into using oso with the [Getting Started guides](getting-started).
{{< /callout >}}

Under the surface, oso is powered by a declarative policy language called Polar.
The Polar language is designed to make simple use cases easy, and complex use cases _possible_.
Expressing "users can see their own data" is as straightforward as

```prolog
allow(user: User, "read", expense: Expense) if
    user = expense.owner;
```

Within the documentation you'll find guides for implementing fine-grained
authorization for everything from multitenant applications, organizational roles,
filesystem-like structures, hierarchical data and more.

{{< callout "Want to know more?" "primary" >}}
  Learn more about oso, the Polar language, and building authorization in
  [Learn oso](../learn/).
{{< /callout >}}
