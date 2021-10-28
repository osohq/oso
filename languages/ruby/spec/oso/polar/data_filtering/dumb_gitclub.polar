actor User {}

resource Org {
  roles = ["owner", "member"];
  permissions = ["read", "create_repos", "list_repos"];

  "read" if "member";
  "list_repos" if "member";

  "create_repos" if "owner";

  "member" if "owner";
}

resource Repo {
  roles = ["reader", "writer"];
  permissions = ["read", "push", "pull", "create_issues", "list_issues"];
  relations = { parent: Org };

  "read" if "reader";
  "pull" if "reader";
  "list_issues" if "reader";

  "push" if "writer";
  "create_issues" if "writer";

  "reader" if "writer";
  "reader" if "member" on "parent";
  "writer" if "owner" on "parent";
}

resource Issue {
  permissions = ["read", "edit"];
  relations = { parent: Repo };
  "read" if "reader" on "parent";
  "edit" if "writer" on "parent";
}

has_role(user: User, name: String, org: Org) if
  role in user.org_roles and
  role matches { name, org_name: org.name };

has_role(user: User, name: String, repo: Repo) if
  role in user.repo_roles and
  role matches { name, repo_name: repo.name };

has_relation(org: Org, "parent", _: Repo{org_name: org.name});
has_relation(repo: Repo, "parent", _: Issue{repo_name: repo.name});

allow(actor, action, resource) if has_permission(actor, action, resource);
