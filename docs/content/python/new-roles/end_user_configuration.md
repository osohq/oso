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
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes.py"
    from="docs: begin-org-role-index"
    to="docs: end-org-role-index"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    hlOpts="hl_lines=4"
    >}}

## Assigning users to a role

Users are assigned to roles using `OsoRoles.assign_role`.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/routes.py"
    from="docs: begin-role-assignment"
    to="docs: end-role-assignment"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    hlOpts="hl_lines=10"
    >}}

{{% callout "Have feedback?" "green" %}}

Have feedback on this documentation or the library itself? It's under
active development. Our engineering team would love to [hear from you in
Slack.](https://join-slack.osohq.com/)

{{% /callout %}}
