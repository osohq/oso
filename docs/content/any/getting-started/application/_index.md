---
title: Add Oso to Your App
weight: 2
description: |
    In this guide, we'll cover the basics of adding Oso to a Python application
    to enforce authorization. Start here to understand the basics of using
    Oso. Then, jump into our more detailed how to section on topics that are
    important to you.
aliases:
  - /getting-started/application/index.html
---

# Add Oso to Your {{% lang %}} App

In this guide, we'll cover the basics of adding Oso to your Python application
to enforce authorization. Start here to understand the basics of using
Oso. Then, jump into our more detailed how to section on topics that are
important to you.

To use Oso, you'll:

- [Model your authorization policy](model) in Polar using *resources*.
  Resources allow you to declaratively specify the permissions and roles
  you want your users to have.
- [Add authorization enforcement](enforce) throughout your app. Call Oso in
  your request handlers to reject or accept requests based on your
  authorization policy.

Depending upon your use case, you may want to:

- [Write Polar rules:](write-rules) Oso can support any authorization model. To
  extend your policy to meet your application's needs you can write Polar
  rules. A Polar rule specifies when a user is allowed to perform a
  specific action on a resource. For example, you may deny access by
  banned users or allow any user to access a public resource.
- [Filter collections of data:](filter-data) Many applications perform
  authorization over large collections of data that cannot be loaded into
  memory. Often index pages showing users a number of resources, like the
  repositories they can access, will need to use data filtering. The *data
  filtering API* provides support for these use cases.
