---
title: Authorize in the UI
weight: 3
description: |
  Control what you expose in the UI with Oso policies.
aliases:
  - /getting-started/ui/index.html
---

# Authorize in the UI

## Using Oso to control UI components

The access control rules that you use to authorize data access in your
application's backend often have implications for the frontend as well.
For example, you may want to hide a button that links to another page if
the current user doesn't have access to that page. Or perhaps you'd like
to display an "edit" button only if the user is actually allowed to edit
the resource in question.

These are examples of what we at Oso call "Authorization-Dependent UI
Elements." In this guide we'll explain how you can use Oso to implement
these kinds of features in your app.

{{% callout "Note" "blue" %}}
We don't currently provide a version of Oso that runs in the
browser. This guide covers how to query for information in the backend
that can be sent to your frontend service.
{{% /callout %}}

### Getting a user's allowed actions

If you're familiar with Oso, you know that Oso policies contain `allow`
rules that specify that an **actor** is allowed to take an **action** on
a **resource.**

In many cases, like when you are authorizing a specific request in the
backend, you'll just want to know if a specific
`(actor, action, resource)` combination is allowed, e.g., is
`alice@gmail.com` allowed to `"READ"` `Expense{id: 1}`? For these yes/no
queries, we provide the `Oso.is_allowed()` method.

But when you're deciding what to display to Alice in the frontend, you
may want more information. It can be especially useful to know **all the
actions** that Alice is allowed to take on the `Expense{id: 1}`
resource, not just whether or not she can read it. In this case, you can
use the `Oso.get_allowed_actions()` method to get a list of Alice's
allowed actions and return them to your frontend.

Let's look at an example. Imagine we have a GitHub-like app that gives
users access to repositories. The page to view a specific repository
looks like this:

![image](python/guides/ui/a.png)

On this page there are several components that we may want to control
based on the actions the current user is allowed to take. For example,
we may want to hide the "Manage Access" or "Delete Repository" links
depending on whether the user is allowed to take those actions.

In this situation, the `Oso.get_allowed_actions()` method can be very
helpful. The method returns a list of actions that a user is allowed to
take on a specific resource. In our example, we call
`Oso.get_allowed_actions()` in the route handler for the "Show
repository" view to get the user's allowed actions for the current repo:

```python
def repos_show(org_id, repo_id):
    # Get repo
    repo = Repository.query.get(repo_id)

    # Authorize repo access
    current_app.oso.authorize(repo, action="READ")

    # Get allowed actions on the repo
    actions = current_app.base_oso.get_allowed_actions(
        get_current_user(), repo
    )
    # Send allowed actions to template (or frontend)
    return render_template(
        "repos/show.html",
        repo=repo,
        org_id=org_id,
        actions=actions,
    )
```

In our demo app, when we call `Oso.get_allowed_actions()` with the user
`mike@monsters.com`, we get back:

```python
actions = ['READ', 'LIST_ROLES', 'CREATE', 'DELETE', 'LIST_ISSUES']
```

But when we call with a different user, `sully@monsters.com`, we get:

```python
actions = ['READ', 'CREATE', 'LIST_ISSUES']
```

The allowed actions for each user are determined by the **Oso policy.**
In this case, our policy has the following rules:

```python
# Repository Permissions
# ----------------------

# Repository members can read and list issues for the repo
allow(user: User, action: String, repo: Repository) if
    repo.is_member(user) and
    action in ["READ", "LIST_ISSUES"];

# Repository admins can list roles and delete the repo
allow(user: User, action: String, repo: Repository) if
    repo.is_admin(user) and
    action in ["LIST_ROLES", "DELETE"];

# Members of the parent organization can create new repos
allow(user: User, "CREATE", repo: Repository) if
    repo.organization.is_member(user);
```

The users Mike and Sully have the following attributes:

- Mike and Sully are both members of the parent organization (Monsters
  Inc.), so they can both create repositories in the organization
- Mike is the admin of the "Paperwork" repository, so he can list
  roles and delete the repo, in addition to reading and listing issues
- Sully is a member of the "Paperwork" repository, so he can only read
  the repo and list issues

Based on these user attributes and our policy, we can see why Mike is
allowed to take more actions on the repository than Sully.

With this relatively straightforward policy, it's easy to trace where
the users' allowed actions come from. But `Oso.get_allowed_actions()`
can be especially powerful with more complicated policies. For example,
if we used Oso's [SQLAlchemy Roles library
features](guides/roles/sqlalchemy_roles),
we could have a policy that looks like this instead:

```python
# Repository Permissions
# ----------------------

# Members of the parent organization can create new repos
role_allow(_role: OrganizationRole{name: "MEMBER"}, "CREATE", _repo: Repository);

# Users with the "READ" role can read and list issues for the repo
role_allow(_role: RepositoryRole{name: "READ"}, action: String, _repo: Repository) if
    action in ["READ", "LIST_ISSUES"];

# Users with the "ADMIN" role can list roles and delete the repo
role_allow(_role: RepositoryRole{name: "ADMIN"}, action: String, _repo: Repository) if
    action in ["LIST_ROLES", "DELETE"];

# Role Hierarchies
# ----------------

# Specify repository role order (most senior on left)
repository_role_order(["ADMIN", "MAINTAIN", "WRITE", "TRIAGE", "READ"]);
```

Now the users' allowed actions depend on their assigned roles for both
the repository and the parent organization, as well as the hierarchy of
the repository roles (for more information on implementing RBAC with
Oso, [check out our
guide](/learn/roles)).

Even with this more complicated policy, we'll still get the correct
allowed actions for Mike and Sully.

### Using allowed actions in the frontend

Since Mike has permission to "LIST_ROLES" and "DELETE" the repo, he
should be able to see the "Manage Access" and "Delete" buttons, but
Sully should not. We can implement this with a simple check in our
template:

```python
{% if "LIST_ROLES" in actions %}
<div>
  <a href={{ url_for('routes.repo_roles_index', org_id=org_id, repo_id=repo.id) }}>
    <h4 class="text-primary">
      <b>
        <pre>Manage Access</pre>
      </b>
    </h4>
  </a>
</div>
{% endif %}
{% if "DELETE" in actions %}
<br />
<form action={{ url_for('routes.repos_show', org_id=org_id, repo_id=repo.id) }} method="POST">
  <button class="btn btn-primary" type="submit" name="delete_repo" value="">
    Delete Repository
  </button>
</form>
{% endif %}
```

Now when Sully logs in, the buttons are hidden:

![image](python/guides/ui/b.png)

Our example uses Flask templates for the UI, but the allowed actions
could be sent to the frontend to make UI decisions in React, Vue or any
other client UI framework.
