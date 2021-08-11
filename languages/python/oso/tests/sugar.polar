allow(actor, action, resource) if
  permission(actor, action, resource);

role(actor, role, resource: Org) if
  actor.has_role_for_resource(name: role, resource: resource);

Org {
  roles = ["owner", "member"];
  permissions = ["invite", "create_repo"];

  "member" if "owner";

  "invite" if "owner";
  "create_repo" if "member";
}
