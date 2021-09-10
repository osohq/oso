allow(actor, action, resource) if
  has_permission(actor, action, resource);

has_role(user: User, name: String, resource: Resource) if
  role in user.roles and
  role matches { name: name, resource: resource };

actor User {}

resource Organization {
  roles = [ "owner" ];
}

resource Repository {
  permissions = [ "read", "push" ];
  roles = [ "contributor", "maintainer" ];
  relations = { parent: Organization };

  "read" if "contributor";
  "push" if "maintainer";

  "contributor" if "maintainer";

  "maintainer" if "owner" on "parent";
}

has_relation(organization: Organization, "parent", repository: Repository) if
  organization = repository.organization;
