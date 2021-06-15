---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso-oso 0.13.0` NEW_VERSION

### Ruby (e.g., 'Core' or 'Python' or 'Node.js')

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



##### Breaking change 1

Summary of breaking change.

Link to [migration guide]().

#### New features

##### Feature 1

Summary of user-facing changes.

Link to [relevant documentation section]().

#### Other bugs & improvements

- Bulleted list
- Of smaller improvements
- Potentially with doc [links]().
