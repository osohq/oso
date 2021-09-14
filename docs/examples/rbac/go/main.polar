# docs: begin-allow
allow(actor, action, resource) if
  has_permission(actor, action, resource);
# docs: end-allow

# docs: begin-has_role
has_role(user: User, name: String, resource: Resource) if
  role in user.Roles and
  role.Name = name and
  role.Resource = resource;
# docs: end-has_role

# docs: begin-actor
actor User {}
# docs: end-actor

resource Organization {
  roles = [ "owner" ];
}

resource Repository {
  permissions = [ "read", "push" ];
  roles = [ "contributor", "maintainer" ];
  # docs: begin-relations
  relations = { parent: Organization };
  # docs: end-relations

  # An actor has the "read" permission if they have the "contributor" role.
  "read" if "contributor";
  # An actor has the "push" permission if they have the "maintainer" role.
  "push" if "maintainer";

  # An actor has the "contributor" role if they have the "maintainer" role.
  "contributor" if "maintainer";

  # An actor has the "maintainer" role if they have the "owner" role on the "parent" Organization.
  "maintainer" if "owner" on "parent";
}

# docs: begin-has_relation
has_relation(organization: Organization, "parent", repository: Repository) if
  organization = repository.Organization;
# docs: end-has_relation
