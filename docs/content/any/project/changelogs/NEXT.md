---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso-oso 0.13.0`

### Ruby

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

The `Query` object returned by `Polar::Polar#query` is now an `Enumerable`.
Previously, you would need to access the `results` attribute which
was an enumerator.

The main impact of this change is that queries are no longer run
on a Fiber, and therefore any methods using Fiber-local variables
(e.g. `Thread.current[:var]`) will work fine.

If you are only using `Oso#allowed?` there is no change needed.

Before:

```ruby
query = oso.query_rule('allow', actor, action, resource)
first = query.results.next
# raises StopIterator if no results
```

After:

```ruby
query = oso.query_rule('allow', actor, action, resource)
first = query.first
# first is nil if there are no results
```

## `sqlalchemy-oso 0.9.0`

### SQLAlchemy (Python)

#### Breaking changes

<!-- TODO: remove warning and replace with "None" if no breaking changes. -->

{{% callout "Warning" "orange" %}}
  This release contains breaking changes. Be sure to follow migration steps
  before upgrading.
{{% /callout %}}

##### Renamed `parent` and `user_in_role` predicates for Role-Based Access Control policies

Two built-in Polar predicates used for implementing [Role-Based Access Control](TODO) have been renamed for
clarity and consistency.

The `parent(child, parent)` predicate has been renamed to `child_parent(child, parent)`.
The `user_in_role(actor, role, resource)` predicate has been renamed to `actor_can_assume_role(actor, role, resource)`.

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
