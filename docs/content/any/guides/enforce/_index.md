---
title: Enforcement
weight: 4
description: |
    Learn how to apply authorization at different layers of your application.
draft: True
---

## Where to apply authorization?

There are a few different where we can apply authorization controls. Applying authorization as early
as possible on the request path can help make sure that every action is authorized which can be a security win. On the other hand, if the decision needs to access application data and context, then it may not be possible.

In this case, keeping the authorization decision as close as possible to the _data_ can ensure that all access to data is done in a secure way, while additionally making available rich context for decisions.