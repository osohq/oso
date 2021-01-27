---
title: Data filtering
weight: 5
description: |
    Efficiently enforce authorization over entire collections
    using oso data filtering
---

## Data Filtering

oso supports applying authorization logic at the ORM layer so that you can
efficiently authorize entire data sets. For example, suppose you have millions
of posts in a social media application created by thousands of users, and
regular users are only authorized to view posts from their friends. It would be
inefficient to fetch all of the posts and authorize them one by one. It would
be much more efficient to distill from the policy a *filter* that can be
applied by the ORM to return only the authorized posts. This idea can be used
in any scenario where you need to authorize a subset of a large collection of
data.

The oso policy engine can now produce such filters from your policy. Below
we’ll briefly explain how it works and link to instructions and examples for
the supported ORMs (currently Django & SQLAlchemy).

* [Django Adapter](./django)
* [SQLAlchemy Adapter](./sqlalchemy)

## How it works

Imagine the following authorization rule. A user is allowed to view any public
social media posts as well as their own private posts:

```polar
allow(user, "view", post) if
    post.access_level = "public" or
    post.creator = user;
```

For a particular user, we can ask two fundamental questions in the context of
the above rule:

1. Is that user allowed to view a specific post, say, `Post{id: 1}`?
2. Which posts is that user allowed to view?

The answer to the first question is a boolean. The answer to the second is a
set of *constraints* that must hold in order for *any* `Post` to be authorized.

oso can produce such constraints through *partial evaluation* of a policy.
Instead of querying with concrete object (e.g., `Post{id: 1}`), you can pass a
`Partial` value, which signals to the engine that constraints should be
collected for it. A successful query for a `Partial` value returns constraint
expressions:

```polar
_this.access_level = "public" or _this.creator.id = 1
```

Partial evaluation is a generic capability of the oso engine, but making use of
it requires an adapter that translates the emitted constraint expressions into
ORM filters. Our first two supported adapters are for the
[Django](getting-started/data-filtering/django) and
[SQLAlchemy](getting-started/data-filtering/sqlalchemy) ORMs, with more on the
way.

These adapters allow oso to effectively translate policy logic into SQL `WHERE`
clauses:

```sql
WHERE access_level = "public" AND creator.id = 1
```

In effect, authorization is being enforced by the policy engine and the ORM
cooperatively.

![Oso data filtering component diagram](img/list-filtering.svg)

## Alternative solutions

Partial evaluation is not the only way to efficiently apply authorization to
collections of data. <!-- TODO(gj): this page doesn't yet exist in the new docs.
On the [Access Patterns]() page, we describe [several alternatives](). -->
Manually applying `WHERE` clauses to reduce the search space (or using
[ActiveRecord-style
scopes](https://guides.rubyonrails.org/active_record_querying.html#scopes))
requires additional application code and still needs to iterate over a
potentially large collection. Authorizing the filter to be applied (or having
oso output the filter) doesn’t require iterating over individual records, but
it does force you to write policy over filters instead of over application
types, which can lead to more complex policies and is a bit of a leaky
abstraction.

## Frameworks

To learn more about this feature and see usage examples, see our ORM specific
documentation:

* [Django](./django)
* [SQLAlchemy](./sqlalchemy)

More framework integrations are coming soon — join us on
[Slack](https://join-slack.osohq.com/) to discuss your use case or open an
issue on [GitHub](https://github.com/osohq/oso).
