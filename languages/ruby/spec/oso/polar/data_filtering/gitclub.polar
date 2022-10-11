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
  relations = { parent: Repo, reviewer: User };
  "read" if "reader" on "parent";
  "edit" if "writer" on "parent";
  "edit" if "reviewer";
}

has_role(_: User{org_roles}, name: String, org: Org) if
  r in org_roles and r matches { name, org };

has_role(_: User{repo_roles}, name: String, repo: Repo) if
  r in repo_roles and r matches { name, repo };

has_relation(org: Org, "parent", _: Repo{org});
has_relation(repo: Repo, "parent", _: Issue{repo});
has_relation(reviewer: User, "reviewer", _: Issue{reviewer});


allow(actor, action, resource) if has_permission(actor, action, resource);
