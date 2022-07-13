---
title: Authorize Across Services
weight: 10
description: |
  If you want to start diving into how to handle authorization in two or
  thousands of services learn more about Oso Cloud. Our latest creation
  makes authorization across services as easy as oso.authorize(user,
  action, resource).
---

# Authorize Across Services

If you're performing authorization in multiple services, you'll need a way to share
authorization data (like roles or resource groups) between your applications.
Oso's authorization-as-a-service product, called [Oso Cloud](https://cloud-docs.osohq.com/), lets you store
authorization data and perform authorization from any of your applications. Like
the Oso Library, Oso Cloud is powered by [the Polar language](https://cloud-docs.osohq.com/reference/polar-syntax).

<img src="basic-architecture.png" class="block mx-auto" style="max-width: 600px" />

Here are some other resources that might be useful:

- [How Oso Cloud Works](https://cloud-docs.osohq.com/concepts/how-it-works): a
  high-level overview of how Oso Cloud enforces authorization in all of your services.
- [Authorization Academy Chapter
  VI](https://www.osohq.com/academy/microservices-authorization) discusses
  how to build an authorization system that works across multiple services.
- [Patterns for Authorization in
  Microservices](https://www.osohq.com/post/microservices-authorization-patterns):
  from the Oso engineering blog, a discussion of the most common patterns we've
  seen for modeling authorization data in a microservices environment.
