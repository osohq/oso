---
title: Exposing authorization configuration to end-users
weight: 5
description: >
  Allow your users to assign roles and change access levels
---

# Exposing authorization to end-users

Most implementations of role-based access control require some level of
end user configuration. In this guide, we'll see how to use the role
management API to allow end users to build authorization.

## Retrieving roles for a resource

Roles can be displayed by using the roles read API.

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/routes/role_assignments.py"
    from="docs: begin-org-role-index"
    to="docs: end-org-role-index"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
    hlFrom="docs: begin-org-role-index-highlight"
    hlTo="docs: end-org-role-index-highlight"
    >}}

## Assigning users to a role

Users are assigned to roles using `OsoRoles.assign_role`.

{{< literalInclude
    path="examples/gitclub/backends/flask-sqlalchemy/app/routes/role_assignments.py"
    from="docs: begin-role-assignment"
    to="docs: end-role-assignment"
    gitHub="https://github.com/osohq/gitclub"
    linenos=true
    hlFrom="docs: begin-role-assignment-highlight"
    hlTo="docs: end-role-assignment-highlight"
    >}}

## Have feedback?

If at any point you get stuck, drop into our
[Slack](https://join-slack.osohq.com/) or <a href="mailto:engineering@osohq.com">send an email</a> to our engineering
team and we'll unblock you.
