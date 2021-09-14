actor User {}

resource Organization {
	permissions = ["add_member", "read", "delete"];
	roles = ["member", "owner"];

	"add_member" if "owner";
	"delete" if "owner";

	"member" if "owner";
}

# Anyone can read.
allow(_, "read", _org: Organization);

resource Repository {
	permissions = ["read", "push", "delete"];
	roles = ["contributor", "maintainer", "admin"];
	relations = { parent: Organization };

	"read" if "contributor";
	"push" if "maintainer";
	"delete" if "admin";

	"maintainer" if "admin";
	"contributor" if "maintainer";

	"contributor" if "member" on "parent";
	"admin" if "owner" on "parent";
}

has_relation(organization: Organization, "parent", repository: Repository) if
	repository.organization = organization;

has_role(user: User, role_name: String, repository: Repository) if
	role in user.repo_roles and
	role.name = role_name and
	role.repo_id = repository.id;

has_role(user: User, role_name: String, organization: Organization) if
	role in user.org_roles and
	role.name = role_name and
	role.org_id = organization.id;

allow(actor, action, resource) if has_permission(actor, action, resource);