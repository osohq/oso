allow(actor, action, resource) if
  has_permission(actor, action, resource);

resource Repo {
  roles = [ "writer", "reader" ];
  permissions = [ "push", "pull" ];
  relations = { parent: Org };

  "push" if "writer";
  "pull" if "reader";

  "reader" if "writer";

  "writer" if "owner" on "parent";
}

has_role(user: User, role_name, resource) if
  role in user.roles and
  role.name = role_name and
  role.resource = resource;

has_relation(org: Org, "parent", repo: Repo) if
  org = repo.org;

resource Org {
  roles = [ "owner" ];
}
