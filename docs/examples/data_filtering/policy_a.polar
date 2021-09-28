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

allow(actor, action, resource) if has_permission(actor, action, resource);

has_role(user: User, role_name: String, repository: Repository) if
	role in user.repo_roles and
	role.name = role_name and
	role.repo_id = repository.id;