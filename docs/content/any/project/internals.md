---
title: Internals
weight: 7
aliases:
    - ../more/internals.html
---

# Internals

Oso is supported in a number of languages,
but the [Oso core](https://github.com/osohq/oso) is written in Rust,
with bindings for each specific language.

At the core of Oso is the **Polar language**. This handles parsing
policy files and executing queries in the form of a virtual machine. Oso was
designed from the outset to be natively embedded in different
languages. It exposes a foreign function interface (FFI) to allow the calling
language to drive the execution of its virtual machine.

Oso can read files with the `.polar` suffix, which are policy files written in Polar syntax.
These are parsed and loaded into a *knowledge base*, which can be thought of an
in-memory cache of the rules in the file.

Applications using Oso can tell it relevant information, for example registering
classes to be used with policies, which are similarly stored in the knowledge base.
The Oso implementation can now be seen as a bridge between the policy code and the application classes.

The Oso library is responsible for converting types between Oso primitive types
(like strings, numbers, and lists), and native application types (e.g. Python’s
`str`, `int`, and `list` classes), as well as keeping track of instances
of application classes.

When executing a query like `oso.query("allow", [user,
"view", expense])` Oso creates a new virtual machine to execute the query.
The virtual machine executes as a coroutine with the native library, and
therefore your application. To make authorization decisions, your application
asks Oso a question: is this (actor, action, resource) triple allowed? To answer
the question, Oso may in turn ask questions of your application: What’s the
actor’s name? What’s their organization? What’s the resource’s id? etc. The
library provides answers by inspecting application data, and control passes back
and forth until the dialog terminates with a final “yes” or a “no” answer to the
original authorization question. The virtual machine halts, and the library
returns the answer back to your application as the authorization decision.


## Data Filtering

Oso supports applying authorization logic at the ORM layer so that you can
efficiently authorize entire data sets. For example, suppose you have millions
of posts in a social media application created by thousands of users, and
regular users are only authorized to view posts from their friends. It would be
inefficient to fetch all of the posts and authorize them one by one. It would
be much more efficient to distill from the policy a _filter_ that can be
applied by the ORM to return only the authorized posts. This idea can be used
in any scenario where you need to authorize a subset of a large collection of
data.

The Oso policy engine can now produce such filters from your policy.

### How it works

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
set of _constraints_ that must hold in order for _any_ `Post` to be authorized.

Oso can produce such constraints through _partial evaluation_ of a policy.
Instead of querying with concrete object (e.g., `Post{id: 1}`), you can pass a
`Partial` value, which signals to the engine that constraints should be
collected for it. A successful query for a `Partial` value returns constraint
expressions:

```polar
_this.access_level = "public" or _this.creator.id = 1
```

Partial evaluation is a generic capability of the Oso engine, but making use
of it requires an adapter that translates the emitted constraint expressions
into ORM filters. Our first two supported adapters are for the [Django]({{<
relref path="reference/frameworks/data_filtering/django" lang="python" >}}) and
[SQLAlchemy]({{< relref path="reference/frameworks/data_filtering/sqlalchemy" lang="python"
>}}) ORMs, with more on the way.

These adapters allow Oso to effectively translate policy logic into SQL `WHERE`
clauses:

```sql
WHERE access_level = "public" OR creator.id = 1
```

In effect, authorization is being enforced by the policy engine and the ORM
cooperatively.

![Oso data filtering component diagram](img/list-filtering.svg)

### Alternative solutions

Partial evaluation is not the only way to efficiently apply authorization to
collections of data. <!-- TODO(gj): this page doesn't yet exist in the new docs.
On the [Access Patterns]() page, we describe [several alternatives](). -->
Manually applying `WHERE` clauses to reduce the search space (or using
[ActiveRecord-style
scopes](https://guides.rubyonrails.org/active_record_querying.html#scopes))
requires additional application code and still needs to iterate over a
potentially large collection. Authorizing the filter to be applied (or having
Oso output the filter) doesn’t require iterating over individual records, but
it does force you to write policy over filters instead of over application
types, which can lead to more complex policies and is a bit of a leaky
abstraction.

### Frameworks

To learn more about this feature and see usage examples, see our ORM specific
documentation:

- [Filter Collections with Django]({{< relref path="reference/frameworks/data_filtering/django" lang="python" >}})
- [Filter Collections with SQLAlchemy]({{< relref path="reference/frameworks/data_filtering/sqlalchemy" lang="python" >}})

More framework integrations are coming soon — join us on
[Slack](https://join-slack.osohq.com/) to discuss your use case or open an
issue on [GitHub](https://github.com/osohq/oso).
