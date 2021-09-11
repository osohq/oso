allow(actor, action, resource) if
  has_permission(actor, action, resource);

has_role(user: User, name: String, resource: Resource) if
  role in user.roles and
  role.name = name and
  role.resource = resource;

actor User {}

resource Organization {
  roles = [ "owner" ];
}

resource Repository {
  permissions = [ "read", "push" ];
  roles = [ "contributor", "maintainer" ];
  relations = { parent: Organization };

  # An actor has the "read" permission if they have the "contributor" role.
  "read" if "contributor";
  # An actor has the "push" permission if they have the "maintainer" role.
  "push" if "maintainer";

  # An actor has the "contributor" role if they have the "maintainer" role.
  "contributor" if "maintainer";

  # An actor has the "maintainer" role if they have the "owner" role on the "parent" Organization.
  "maintainer" if "owner" on "parent";
}

has_relation(organization: Organization, "parent", repository: Repository) if
  organization = repository.organization;
