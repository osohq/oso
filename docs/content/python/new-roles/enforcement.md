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
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes.py"
    from="docs: begin-is-allowed"
    to="docs: end-is-allowed"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

## Enforcing authorization to a collection of resources

Certain API endpoints involving batch operations 

## Enforcing authorization on data mutations

## Varying our user interface depending on authorization
