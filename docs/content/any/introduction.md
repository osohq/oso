---
title: Introduction
any: true
hideContents: true
draft: true
---

### What is Oso?

Oso is an **open source library for authorization**. You use it to define what
users can and cannot do in your application.

Oso gives you a structured way to implement concepts like
"users can see their own data", or to fast-track adding common access control
patterns like role-based access control.

{{% callout "Ready to go?" "primary" %}}
  Dive straight into using Oso with the [Quickstart](quickstart).
{{% /callout %}}

Under the surface, Oso is powered by a declarative policy language called
Polar. The Polar language is designed to make simple use cases easy, and
complex use cases _possible_. Expressing "users can see their own data" is as
straightforward as:

```polar
allow(user: User, "read", expense: Expense) if
    user = expense.owner;
```

Within the documentation you'll find guides for implementing fine-grained
authorization for everything from multitenant applications, organizational
roles, filesystem-like structures, hierarchical data and more.

{{% callout "Want to know more?" "primary" %}}
  Learn more about Oso, the Polar language, and building authorization in
  [Learn Oso](../learn/).
{{% /callout %}}
