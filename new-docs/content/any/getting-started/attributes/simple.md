---
title: Users can see their own data
weight: 1
description: |
    Implement fine-grained authorization by expressing who can see what data
    based on who they are and their relationship to the data.
---

{{% coming_soon %}}

TODO. Basically:

```prolog
allow(user, "read", data) if user = data.owner;
```