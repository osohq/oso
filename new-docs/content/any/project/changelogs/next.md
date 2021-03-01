---
title: Release 2021-02-XX
menuTitle: 2021-02-XX
any: true
description: Changelog for Release 2021-02-XX (sqlalchemy-oso 0.5.1) bug fixes.
draft: true
---

## `oso` 0.X.X

### Improved performance for data filtering

Data filtering performance has been improved with some queries
experiencing a 5x speed up.

## `sqlalchemy-oso` 0.5.1

Fixed a bug in `get_resource_users_by_role` which meant it would only work
if the roles were defined for a resource called "repository".

Many thanks to [Sascha Jullmann](https://github.com/saschajullmann) for
[reporting](https://github.com/osohq/oso/issues/740) and
[fixing](https://github.com/osohq/oso/pull/745) the bug.
