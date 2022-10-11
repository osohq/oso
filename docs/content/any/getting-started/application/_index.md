---
title: Add Oso to Your App
metaTitle: Add Oso to Your $LANG App
weight: 2
description: |
    In this guide, we'll cover how to add Oso to your application.
    Start here to understand how to use Oso.
aliases:
  - /getting-started/application/index.html
---

# Add Oso to Your {{% lang %}} App

In this guide, we'll cover how to add Oso to your {{% lang %}} application.
Start here to understand how to use Oso. Then, jump into our
more detailed how to section on topics that are
important to you.

To use Oso, you'll:

- [Model your authorization logic](model) by writing a _policy_ with _resources_.
  Resources allow you to declaratively specify the permissions and roles
  you want your users to have.
- [Enforce authorization](enforce) throughout your app. Call Oso in
  your request handlers to reject or accept requests based on your
  authorization policy.

Once you've added policies and enforcement, you'll have Oso setup and
enforcing authorization in your application. To go further with Oso:

- [Write Polar rules](write-rules) to extend your authorization model with
  custom logic. A Polar rule specifies when a user is allowed to perform a
  specific action on a resource. For example, you may deny access from
  banned users or allow any user to access a public resource.
- [Filter collections of data:](filter-data) by applying enforcement at
  the data access layer. Many applications perform authorization over
  large collections of data that cannot be loaded into memory. Often index
  pages showing users a number of resources, like the repositories they
  can access, will need to use data filtering. The *data filtering API*
  provides support for these use cases.

The Oso Library works best in monolithic applications. If you're building authorization for more than one service or want to share a policy across multiple applications, read how to [add Oso Cloud to your app](https://www.osohq.com/docs/get-started/add-to-your-app).
