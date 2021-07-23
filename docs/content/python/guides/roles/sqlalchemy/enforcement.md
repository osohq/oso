---
title: Enforcing authorization in GitClub
weight: 4
description: >
    How to enforce authorization
---

# Enforcement

In this guide, we'll cover common enforcement tasks in the context of
GitClub.

## Enforcing authorization on a single resource

Most backend web API endpoints deal with authorizing access to a single
resource at a time. In GitClub, some examples of this pattern are the `POST
/orgs` endpoint for creating a single organization, `GET /users/:user_id` for
viewing a single user, and `DELETE /orgs/<int:org_id>/roles` for deleting a
user's role for a particular organization.

`Oso.is_allowed()` is typically the best option for enforcing authorization on
a single resource. Here's how that looks for the `POST /orgs` endpoint:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes/orgs.py"
    from="docs: begin-is-allowed"
    to="docs: end-is-allowed"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

## Enforcing authorization on a collection of resources

Certain API endpoints involve enforcing authorization over a collection of
resources. The primary example of this pattern is an "index" or "list" endpoint
such as `GET /orgs`.

[Data filtering](guides/data_access/sqlalchemy) is the best option for
enforcing authorization on a list endpoint. Here's how that looks for the `GET
/orgs` endpoint:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes/orgs.py"
    from="docs: begin-org-index"
    to="docs: end-org-index"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

Because we're using an `AuthorizedSession` to query for organizations, Oso
evaluates our authorization policy into a set of constraints that are applied
directly to the SQLAlchemy query, preventing unauthorized data from being
loaded from the database. This is more performant than loading *all*
organizations from the database and then filtering out unauthorized rows in a
follow-up pass, and it's also more secure since the unauthorized data never
leaves the database.

## Enforcing authorization on data mutations

For endpoints that mutate data (creating, updating, and deleting resources),
passing the to-be-mutated resource to `Oso.is_allowed()` is the best way to
enforce authorization.

When creating a new resource, we can ensure that the current user is allowed to
create that type of resource by passing the resource to `Oso.is_allowed()`
before persisting it to the database. For example, in GitClub any logged-in
user is allowed to create a new organization, so we have a Polar rule that
indicates exactly that:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-org-create-rule"
    to="docs: end-org-create-rule"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

In the `POST /orgs` route handler, we pass `g.current_user` to
`Oso.is_allowed()` as the actor:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes/orgs.py"
    from="docs: begin-is-allowed"
    to="docs: end-is-allowed"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

If the current user isn't logged in, `g.current_user` will be `None`, which
won't match the `User` specializer in the `allow()` rule, and the user's
request to create a new org will be denied.

When updating or deleting an existing resource, it's best practice to pass the
resource in question to `Oso.is_allowed()` and to write Polar authorization
rules over the resource.

<!-- TODO(gj): example in GitClub -->

## Varying our user interface depending on authorization

Coming soon!

## Have feedback?

If at any point you get stuck, drop into our
[Slack](https://join-slack.osohq.com/) or <a href="mailto:engineering@osohq.com">send an email</a> to our engineering
team and we'll unblock you.
