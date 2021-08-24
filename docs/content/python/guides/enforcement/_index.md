---
title: Enforcing an Oso Policy (Oso 0.20.0 Beta)
weight: 4
any: true
# showContentForAnyLanguage: true
# draft: True
---

{{% callout "Note: 0.20.0 Beta Feature" %}}
  This is an API provided by the beta release of Oso 0.20.0, meaning that it is
  not yet officially released. You may find other docs that conflict with the
  guidance here, so proceed at your own risk! If you have any questions, don't
  hesitate to [reach out to us on Slack](https://join-slack.osohq.com). We're
  here to help.
{{% /callout %}}

<div class="pb-10"></div>

# Use Oso Policies in an App

To use an Oso policy in your app, you'll need to "enforce" it. A policy is
useless without an app that consults the policy on user actions. For most apps,
policies can be enforced on multiple "levels":
  - [Resource-level](resource.html): is the user allowed to perform this action on a particular resource?
  - [Field-level](field.html): which fields on this object can the user read? Which ones can they update?
  - [Request-level](request.html): should this user even be able to hit this endpoint, regardless of the resources it involves?
  <!-- - [Query-level](query.html): fetch all the resources that the user has access to. -->

Oso provides an API to enforce authorization at all levels, each of which are
described in this guide.

To get started, read about [building an enforcer instance](enforcer.html).
