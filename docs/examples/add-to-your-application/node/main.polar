actor User {}

resource Repository {
    permissions = ["read", "push", "delete"];
    roles = ["contributor", "maintainer", "admin"];

    "read" if "contributor";
    "push" if "maintainer";
    "delete" if "admin";

    "maintainer" if "admin";
    "contributor" if "maintainer";
}

has_role(actor: User, role_name: String, repository: Repository) if
    role in actor.roles and
    role_name = role.name and
    repository = role.repository;

allow(actor, action, resource) if
    has_permission(actor, action, resource);
