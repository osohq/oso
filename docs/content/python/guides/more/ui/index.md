---
title: Authorize in the UI
weight: 3
description: |
  Control what you expose in the UI with Oso policies.
aliases:
  - /getting-started/ui/index.html
  - /guides/ui.html
---

# Authorize in the UI

## Using Oso to control UI components

The access control rules that you use to authorize data access in your
application's backend often have implications for the frontend as well.
For example, you may want to hide a button that links to another page if
the current user doesn't have access to that page. Or perhaps you'd like
to display an "edit" button only if the user is actually allowed to edit
the resource in question.

In this guide we'll explain how you can use Oso to implement these kinds of
features in your app.

{{% callout "Note" "blue" %}}
We don't currently provide a version of Oso that runs in the
browser. This guide covers how to query for information in the backend
that can be sent to your frontend service.
{{% /callout %}}

### Getting a user's authorized actions

When you're deciding what to display to a user in the frontend, it can be useful
to know **all the actions** that the user is allowed to take on a
resource, rather than checking if a single action is allowed.
In this case, you can use the {{< apiDeepLink class="Oso"
label="authorized_actions(actor, resource)" >}}authorized_actions{{<
/apiDeepLink >}} method to get a list of a user's authorized actions and return
them to your frontend.

Let's look at an example. Imagine we have a GitHub-like app that gives users
access to repositories. The page to view a specific repository looks like this:

![image](img/ui-a.png)

On this page, we may want to hide the "Issues" or "Settings" links
depending on whether the user is allowed to view issues or adjust repository settings.

In this situation, you can use the {{< apiDeepLink class="Oso"
label="authorized_actions()" >}}authorized_actions{{<
/apiDeepLink >}} method. The method returns a list of actions that a user is authorized to
take on a specific resource. In our example, we can call
`authorized_actions()` in the route handler for the "Show
repository" view to get the user's allowed actions for the current repo:

{{< code file="app.py" hl_lines="9,10,11,17" >}}
def repos_show(org_id, repo_id):
    # Get repo
    repo = Repository.query.get(repo_id)

    # Authorize repo access
    current_app.oso.authorize(repo, action="READ")

    # Get allowed actions on the repo
    actions = current_app.base_oso.authorized_actions(
        get_current_user(), repo
    )
    # Send allowed actions to template (or frontend)
    return render_template(
        "repos/show.html",
        repo=repo,
        org_id=org_id,
        actions=actions,
    )
{{< /code >}}

In our demo app, when we call `Oso.authorized_actions()` with the user
`mike@monsters.com`, we get back:

```python
actions = ['read', 'list_roles', 'delete', 'list_issues']
```

But when we call with a different user, `sully@monsters.com`, we get:

```python
actions = ['read', 'list_issues']
```

The authorized actions for each user are determined by the **Oso policy.**
In this case, our policy has the following rules:

{{< code file="main.polar" >}}
resource Repository {
  permissions=["read", "list_roles", "delete", "list_issues"];
  roles=["contributor", "maintainer", "admin"];

  # Repository contributors can read and list issues for the repo
  "read" if "contributor";
  "list_issues" if "contributor";

  # Repository admins can list roles and delete the repo
  "list_roles" if "admin";
  "delete" if "admin";

  # Repository admins are contributors by default
  "contributor" if "admin";
}

# ...
{{< /code >}}


The users Mike and Sully have the following attributes:

- Mike is the admin of the "Paperwork" repository, so he can list
  roles and delete the repo, in addition to reading and listing issues
- Sully is a member of the "Paperwork" repository, so he can only read
  the repo and list issues

Based on these user attributes and our policy, we can see why Mike is
allowed to take more actions on the repository than Sully.


### Using allowed actions in the frontend

Since Mike has permission to `"list_roles"` and `"delete"` the repo, he
should be able to see the "Manage Access" and "Delete" buttons, but
Sully should not. We can implement this with a simple check in our
template:

```python
{% if "list_roles" in actions %}
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
{% if "delete" in actions %}
<br />
<form action={{ url_for('routes.repos_show', org_id=org_id, repo_id=repo.id) }} method="POST">
  <button class="btn btn-primary" type="submit" name="delete_repo" value="">
    Delete Repository
  </button>
</form>
{% endif %}
```

Now when Sully logs in, the buttons are hidden:

![image](img/ui-b.png)

Our example uses Flask templates for the UI, but the allowed actions
could be sent to the frontend to make UI decisions in React, Vue or any
other client UI framework.
