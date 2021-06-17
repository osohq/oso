---
title: Release YYYY-MM-DD
menuTitle: YYYY-MM-DD
any: true
description: >-
  Changelog for Release YYYY-MM-DD (RELEASED_VERSIONS) containing new features,
  bug fixes, and more.
draft: true
---

## `oso 0.13.0`

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


### Core

#### Breaking changes

Attempting to create a dictionary with a repeated key is now a parser error.
Previously, the first (key, value) pair would be taken and the others would
be dropped.

Before:

```polar
query> d = {a: 1, a: 2}
d = {'a': 1}
```

After:

```polar
query> d = {a: 1, a: 2}
ParserError
Duplicate key: a at line 1, column 6
```

#### Other bugs & improvements

Trailing commas are now supported in dictionaries and lists.
For example:

```polar
allow(_user, action, repository: Repository) if
  action in [
    "read",
    "write",
  ];
```