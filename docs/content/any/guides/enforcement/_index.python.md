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

An `Oso` instance provides the following methods to enforce to make it easy to
enforce your policy in a number of situations:

- {{< apiDeepLink class="Oso" label="authorize(actor, action, resource)"
  >}}authorize{{< /apiDeepLink >}}: Ensure that an actor can perform an action
  on a certain resource. Read about [resource-level enforcement](resource.html).
- {{< apiDeepLink class="Oso" label="authorize_request(actor, request)"
  >}}authorize_request{{< /apiDeepLink >}}:
  Ensure that an actor is allowed to access a certain endpoint. Read about
  [request-level enforcement](request.html).
- {{< apiDeepLink class="Oso" label="authorize_field(actor, action, resource, field)" >}}authorize_field{{< /apiDeepLink >}}:
  Ensure that a actor can perform a particular action on one _field_ of a given
  resource. Read about [field-level enforcement](field.html).
- {{< apiDeepLink class="Oso" label="authorized_actions(actor, resource)" >}}authorized_actions{{< /apiDeepLink >}}:
  List the actions that `actor` is allowed to take on `resource`.
- {{< apiDeepLink class="Oso" label="authorized_fields(actor, action, resource)" >}}authorized_fields{{< /apiDeepLink >}}:
  List the fields that `actor` is allowed to perform `action` upon.


We recommend starting out by reading about [resource-level enforcement](resource.html).
