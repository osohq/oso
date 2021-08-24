allow(actor, action, resource) if
  permission(actor, action, resource);

role(actor, role, resource) if
  actor.has_role_for_resource(name: role, resource: resource);

role(user: User, role, resource) if
  team in user.teams and
  role(team, role, resource);

resource Org {
  roles = ["owner", "member"];
  permissions = ["invite", "create_repo"];
  relations = { owns: User };

  "member" if "owner";

  "invite" if "owner";
  "create_repo" if "member";
}

relation(user: User, "owns", org: Org) if
  user = org.owner;

resource Repo {
  roles = ["reader", "writer", "admin"];
  permissions = ["pull", "push"];
  relations = {
    parent: Org,
  };

  "admin" if "owner" on "parent";
  "admin" if "owns" on "parent";
  "reader" if "member" on "parent";

  "writer" if "admin";
  "reader" if "writer";

  "push" if "writer";
  "pull" if "reader";
}

relation(org, "parent", repo: Repo) if
  org = repo.org and
  org matches Org;

permission(_: User, "pull", repo: Repo) if
  repo.is_public;

resource Issue {
  permissions = ["delete", "edit"];
  relations = {
    parent: Repo,
    creator: User,
  };

  "edit" if "writer" on "parent";
  "delete" if "admin" on "parent";
  "delete" if "creator";
}

relation(repo, "parent", issue: Issue) if
  repo = issue.repo and
  repo matches Repo;

relation(user: User, "creator", issue: Issue) if
  user = issue.creator;
