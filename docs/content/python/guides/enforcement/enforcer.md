---
title: "Build an enforcer"
weight: 1
any: true
description: >
  Learn how to construct an Enforcer instance so that you can use its methods
  throughout your app.
# showContentForAnyLanguage: true
# draft: True
---

{{% callout "Note: 0.20.0 Alpha Feature" %}}
  This is an API provided by the alpha release of Oso 0.20.0, meaning that it is
  not yet officially released. You may find other docs that conflict with the
  guidance here, so proceed at your own risk! If you have any questions, don't
  hesitate to [reach out to us on Slack](https://join-slack.osohq.com). We're
  here to help.
{{% /callout %}}

<div class="pb-10"></div>

# Construct an `Enforcer` instance

To make use of the new Oso enforcement APIs, you'll need an Enforcer instance.
The enforcer is a link between your policy and your application code, and should
be accessible throughout your app.

```python
from oso import Enforcer, Policy

def init_oso(app):
  # Build a policy and configure it
  policy = Policy()
  policy.register_class(...)
  policy.load_file(...)

  oso = Enforcer(policy)

  # Make the oso enforcer accessible throughout the app
  app.oso = oso
```

An enforcer exposes a number of useful methods that your app can use. Each one
queries your policy for `allow`, `allow_request`, or `allow_field` rules. To
learn more about writing those rules, read about [writing a
policy](../../getting-started/policies).

- {{< apiDeepLink class="Enforcer" label="authorize(actor, action, resource)"
  >}}authorize{{< /apiDeepLink >}}: Ensure that an actor can perform an action
  on a certain resource. Read about [resource-level enforcement](resource.html).
- {{< apiDeepLink class="Enforcer" label="authorize_request(actor, request)"
  >}}authorize_request{{< /apiDeepLink >}}:
  Ensure that an actor is allowed to access a certain endpoint. Read about
  [request-level enforcement](request.html).
- {{< apiDeepLink class="Enforcer" label="authorize_field(actor, action, resource, field)" >}}authorize_field{{< /apiDeepLink >}}:
  Ensure that a actor can perform a particular action on one _field_ of a given
  resource. Read about [field-level enforcement](field.html).
- {{< apiDeepLink class="Enforcer" label="authorized_actions(actor, resource)" >}}authorized_actions{{< /apiDeepLink >}}:
  List the actions that `actor` is allowed to take on `resource`.
- {{< apiDeepLink class="Enforcer" label="authorized_fields(actor, action, resource)" >}}authorized_fields{{< /apiDeepLink >}}:
  List the fields that `actor` is allowed to perform `action` upon.
