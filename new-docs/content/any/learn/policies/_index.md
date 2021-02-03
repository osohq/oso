---
title: Writing policies
weight: 2
any: true
aliases: 
    - ../getting-started/policies/index.html
---

# Writing Policies

Policies are the source of truth for the authorization logic used to evaluate
queries in Oso. As a reminder: Oso policies are written in a declarative
language called Polar. There is a full [Polar Syntax guide](polar-syntax) which
you can use as a reference of all the available syntax, but here we’ll give an
overview of getting started with writing policies.

The syntax might feel a bit foreign at first, but fear not: almost anything you
can express in imperative code can equally be expressed in Polar — often more
concisely and closer to how you might explain the logic in natural language.

{{% callout "Note" "green" %}}
  Policies are stored in Polar files (extension `.polar`), which are loaded
  into the authorization engine using the [Oso library](reference).
{{% /callout %}}
