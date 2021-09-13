allow(actor, action, resource) if
  has_permission(actor, action, resource);

actor User {}
actor Team {}

has_role(actor: Actor, role: String, resource: Resource) if
  actor.has_role_for_resource(name: role, resource: resource);

has_role(user: User, role: String, resource: Resource) if
  team in user.teams and
  has_role(team, role, resource);

resource Org {
  roles = ["owner", "member"];
  permissions = ["invite", "create_repo"];
  relations = { owns: User };

  "member" if "owner";

  "invite" if "owner";
  "create_repo" if "member";
}

has_relation(user: User, "owns", org: Org) if
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

has_relation(org: Org, "parent", repo: Repo) if
  org = repo.org;

has_permission(_: User, "pull", repo: Repo) if
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

has_relation(repo: Repo, "parent", issue: Issue) if
  repo = issue.repo;

has_relation(user: User, "creator", issue: Issue) if
  user = issue.creator;
