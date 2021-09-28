---
title: Enforce an Oso Policy
weight: 3
any: true
aliases:
  - ../../../learn/enforce.html
---

# Enforce an Oso Policy

To use an Oso policy in your app, you'll need to "enforce" it. A policy is
useless without an app that consults the policy on user actions. For most apps,
policies can be enforced on multiple "levels":
  - [Resource-level](resource.html): is the user allowed to perform this action on a particular resource?
  - [Field-level](field.html): which fields on this object can the user read? Which ones can they update?
  - [Request-level](request.html): should this user even be able to hit this endpoint, regardless of the resources it involves?
  <!-- - [Query-level](query.html): fetch all the resources that the user has access to. -->

Oso provides an API to enforce authorization at all levels, each of which are
described in this guide.

We recommend starting out by reading about [resource-level enforcement](resource.html).
