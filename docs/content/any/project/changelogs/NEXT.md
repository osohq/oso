---
title: Release 2021-05-DD
menuTitle: 2021-05-DD
any: true
description: >-
  Changelog for Release 2021-05-DD (sqlalchemy-oso 0.6.2) containing new features,
  bug fixes, and more.
draft: true
---

## `sqlalchemy-oso` 0.6.2

### Other bugs & improvements

- Authorized sessions now disable [baked queries][] by default because they can
  lead to authorization backdoors. If you understand the risks and still want
  to opt-in to the previous behavior of using baked queries, you can pass the
  `enable_baked_queries=True` kwarg to `sqlalchemy_oso.authorized_sessionmaker()`
  and friends.

[baked queries]: https://docs.sqlalchemy.org/en/13/orm/extensions/baked.html
