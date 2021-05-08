---
title: Role modeling by example
weight: 3
description: >
    Let's see how to build common role models with Oso Roles.
---

# Role modeling by example

In this guide, we'll cover a few different role models in the context of
the [GitClub example application](https://github.com/osohq/gitclub-sqlalchemy-flask-react).

{{% callout "Read first" "blue" %}}
 - [Getting started](getting-started)
{{% /callout %}}

## Multiple role types: Repository roles

In the [Getting Started](getting-started) guide, we only discussed a role for a single
resource type (`Org`). However, many apps will have multiple levels of roles. It
is common to have roles associated with an organization object, like
"owner" and "member", and more granular roles associated with resources within an organization/tenant (we'll refer to these as "child resources"). In
the GitClub example application, the Organization model has "member" and
"owner" roles. The Repository model has "reader" and "writer" roles.

To define a role for another resource type, we add another `resource`
rule to our policy.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-repo-resource"
    to="docs: end-repo-resource"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

This defines two roles on repository, `"repo_read"` and `"repo_write"`.
This allows us to assign users directly to repositories without
giving them a role in the entire organization.

Now, we can use `OsoRoles.assign_role` to assign role on
repositories in addition to organizations.

## Grant access to child resources with implied roles

Often a role on a parent resource will grant access to all child
resources. In GitClub, a member of an organization is granted read access to
all repositories within that organization.

We can model this access control model with **cross-resource implied
roles**.

First, we'll define how organizations and repositories are related.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-repo-parent"
    to="docs: end-repo-parent"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

This `parent(child, parent)` rule defines that an organization is
considered a parent of a repository if it is assigned to the
repository's `org` field.

When Oso evaluates the policy, it uses the `org` relationship defined
on our model:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/models.py"
    from="docs: begin-repo-model"
    to="docs: end-repo-model"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    hlOpts="hl_lines=9"
>}}

Then, we **imply** a role on the child resource from our parent resource
definition.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-org-resource"
    to="docs: end-org-resource"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    hlOpts="hl_lines=6"
    >}}

The `"repo_read"` entry in `implies: ` for `org_member` gives organization
members the `repo_read` role on all repositories within the
organization.

{{% callout "Parent limitations" "blue" %}}

Currently a `parent` rule body must be of the form:

```polar
parent(child: ChildType, parent: ParentType) if
    child.attribute = parent;
```

This is sufficient to model resources with single parents.

{{% /callout %}}

## Cross-resource permissions

Sometimes, we may want to grant permissions on child resources but do
not need that resource to have its own roles.

An example in GitClub is issues. Repository writers can read issues, but
repository readers cannot.

First, we define the issue resource in our policy. Even though an issue
doesn't have roles, we still define an issue resource to declare the actions users can take on issues.

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-issue-resource"
    to="docs: end-issue-resource"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
    >}}

Notice the last argument to `resource` is `_` instead of `roles`. This
indicates it is unused.

Now, we can assign the issue action to a repository role. Notice the `"issue:read"` permission for `repo_write` below:

{{< literalInclude
    path="examples/gitclub-sqlalchemy-flask-react/backend/app/authorization.polar"
    from="docs: begin-repo-resource"
    to="docs: end-repo-resource"
    gitHub="https://github.com/osohq/gitclub-sqlalchemy-flask-react"
    linenos=true
>}}

The `identifier:action_name` string is used to identify actions on other
resources when specifying a permission on a role. `identifier` is the
second argument to the `resource` definition.

{{% callout "Have feedback?" "green" %}}

Have feedback on this documentation or the library itself? It's under
active development. Our engineering team would love to [hear from you in
Slack.](https://join-slack.osohq.com/)

{{% /callout %}}
