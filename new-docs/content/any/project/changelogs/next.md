---
title: Release 2021-02-XX
menuTitle: 2021-02-XX
any: true
description: Changelog for Release 2021-02-XX (sqlalchemy-oso 0.5.1) bug fixes.
draft: true
---

## `sqlalchemy-oso` 0.5.1

Fixed a bug in `get_resource_users_by_role` which meant it would only work
if the roles were defined for a resource called "repository".

Many thanks to [Sascha Jullmann](https://github.com/saschajullmann) for
[reporting](https://github.com/osohq/oso/issues/740) and
[fixing](https://github.com/osohq/oso/pull/745) the bug.


## `oso` 0.12.0

### External Comparison in Javascript

Added external comparison operators to Javascript.  Note that you almost
certainly need to (supply a custom equality function to the Polar constructor)[https://docs.osohq.com/v1/js/node/interfaces/types.options.html]
to use this, as the default
behavior for javascript only is equal if they're the same object.
