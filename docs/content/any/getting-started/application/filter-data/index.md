---
title: Filter collections of data
description: |
  Many applications perform authorization over large collections of data
  that cannot be loaded into memory. Often index pages showing users a
  number of resources, like the repositories they can access, will need to
  use data filtering. The data filtering API provides support for these
  use cases.
weight: 4
---

# Filter collections of data

Many applications perform authorization over large collections of data
that cannot be loaded into memory. Often index pages showing users a
number of resources, like the repositories they can access, will need to
use data filtering. The data filtering API provides support for these
use cases seamlessly without requiring you to alter your policy.

## Get all authorized resources

In [Enforce authorization](enforce) we discussed
*resource-level authorization*. The `authorize` method tells you whether
*a specific resource* is authorized. But to fetch *all* authorized resources, we
need to use `authorized_resources` instead:

{{< literalInclude
    dynPath="dataFilteringPath"
    fallback="todo"
    from="docs: begin-list-route"
    to="docs: end-list-route"
    >}}

To use this API, you must pass some additional information to
`register_class` so that Oso knows how to retrieve your
application's objects.

TODO make sure this matches data filtering guides

## Implementing data filtering query functions

To use data filtering, you tell Oso how to query your data store for the
resources used in your policy. Oso uses **query objects** to query your
data store. A **query object** represents a set of filters to apply to a
collection of data.

You can use any type as a **query object**. Many ORMs have [these built
in]({{% exampleGet "queryObjectExampleLink" %}}), but you may have your
own representation if your resources are retrieved from an external
service, or with a lower-level database API.

You implement three functions to tell Oso how to work with your **query objects**:

- `build_query(filters) -> Query`: Creates a query from a list of authorization filters
  produced by evaluting the Oso policy.
- `exec_query(query) -> List[Object]`: Executes the query, returning
  the list of objects retrieved by the query.
- `combine_query(q1, q2) -> Query`: Combines two queries `q1` and `q2` together such
  that the new query returns the UNION of `q1` and `q2` (all results
  from each).

{{< literalInclude
    dynPath="dataFilteringPath"
    fallback="todo"
    from="docs: begin-data-filtering"
    to="docs: end-data-filtering"
    >}}

When you call `authorized_resources`, Oso will create a query using the
`build_query` function with filters obtained by running the policy. For
example, in [Write Polar Rules](write-rules) we wrote the rule:

```polar
has_permission(_user: User, "read", repository: Repository) if
	repository.is_public = true;
```

This rule would produce the filters: `[Filter(kind=Eq,
field="is_public", value=true)]`. Oso then uses SQLAlchemy in our
example to create a query and retrieve repositories that have the
`is_public` field as `true` from the database by calling the
`exec_query` function. This pushes down filters to the database,
allowing you to retrieve only authorized objects.
Notably, the same rule can be executed using `authorize` and
`authorized_resources`.

## Adding filters on top of authorization

Often, you may want to add to the query after it is authorized. Let's
say we want to order Repositories by name.

To do this, we can use the `authorized_query` API:

<!-- manually test this snippet -->

{{% exampleGet "repoListQuerySnippet" %}}

`authorized_query` returns the query object used by our ORM with
authorization filters applied so that we can add additional filters,
pagination, or ordering to it.

## What's next

In this brief example we covered what the *data filtering API* does. For
a more detailed how to of using data filtering and implementing query
builder functions, see: [How to: Filter data](/guides/data_access).

TODO update title as needed here

This is the end of __Add to your app__! For more detail on using
Oso, see the [How to guides](/guides).
