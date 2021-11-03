allow(actor, action, resource) if
  has_permission(actor, action, resource);

has_role(user: User, name: String, resource: Resource) if
  role in user.roles and
  role.name = name and
  role.resource = resource;

actor User {}

resource Org {
  roles = [ "owner", "member" ];
  permissions = [ "invite", "create_repo" ];

  "create_repo" if "member";
  "invite" if "owner";

  "member" if "owner";
}

resource Repo {
  roles = [ "writer", "reader" ];
  permissions = [ "push", "pull" ];
  relations = { parent: Org };

  "pull" if "reader";
  "push" if "writer";

  "reader" if "writer";

  "reader" if "member" on "parent";
  "writer" if "owner" on "parent";
}

has_relation(org: Org, "parent", repo: Repo) if
  org = repo.org;

resource Issue {
  permissions = [ "edit" ];
  relations = { parent: Repo };

  "edit" if "writer" on "parent";
}

has_relation(repo: Repo, "parent", issue: Issue) if
  repo = issue.repo;
