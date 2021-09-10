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
use cases.

## Authorizing queries

In [Add authorization enforcement](enforce) we discussed enforcing
*resource-level authorization*. The `authorize` method tells you whether
*a specific resource* is authorized. But what about endpoints that need
to authorize a collection of resources?

You can authorize queries to obtain a set of resources that are
authorized for a particular user and action. Let's go back to our
repository example, but this time we are listing repositories the user
has access to:

{{< literalInclude
    path="examples/add-to-your-application/python/app/data_filtering.py"
    from="docs: begin-list-route"
    to="docs: end-list-route"
    >}}

To use this API, you must pass some additional information to
register_class so that Oso knows how to query for your application's
objects.

TODO make sure this matches data filtering guides

## Implementing data filtering query functions

To use data filtering, you tell Oso how to query for the objects in your
policy. You can use another service, an ORM, or a lower level database
API to query objects—the same API you would use in the rest of your
application. You implement three functions:

- `build_query`: Create a query from a list of authorization filters
  produced by the policy.
- `exec_query`: Execute the query.
- `combine_query`: Combine two queries `q1` and `q2` together such
  that the new query returns the UNION of `q1` and `q2` (all results
  from each).

{{< literalInclude
    path="examples/add-to-your-application/python/app/data_filtering.py"
    from="docs: begin-data-filtering"
    to="docs: end-data-filtering"
    >}}

When you call authorized_resources, Oso will create a query using the
`build_query` function with filters obtained by running the policy. For
example, in [Write Polar Rules](write-rules) we wrote the rule:

```python
has_permission(_user: User, "read", repository: Repository) if
	repository.is_public = true;
```

This rule would produce the filters: `[Filter(kind=Eq,
field="is_public", value=true)]`. Oso then uses SQLAlchemy in our
example to create a query and retrieve repositories that have the
`is_public` field as `true` from the database. This pushes down filters
to the database, allowing you to retrieve only authorized objects.
Notably, the same rule can be executed using `authorize` and
`authorized_resources`.

## Adding filters on top of authorization

Often, you may want to add to the query after it is authorized. Let's
say we want to order queries by name.

To do this, we can use the `authorized_query` API:

<!-- manually test this snippet -->

```python
@app.route("/repos")
def repo_list():
    query = oso.authorized_query(
        User.get_current_user(),
        "read",
        Repository)

    # Use the ORM's Query API to alter the query before it is
    # executed by the database with .all().
    repositories = query.order_by(Repository.name).all()

    return serialize(repositories)
```

`authorized_query` returns the query object used by our ORM with
authorization filters applied so that we can add additional filters,
pagination, or ordering to it.

## What's next

In this brief example we covered what the *data filtering API* does. For
a more detailed how to of using data filtering and implementing query
builder functions, see: [How to: Filter data](/guides/data_access).

This is the end of Add to your app! For more detail on using
Oso, see the [How to guides](/guides).
