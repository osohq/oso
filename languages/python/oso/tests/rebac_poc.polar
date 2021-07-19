
# role(user: User, role: String, resource: OsoResource) if user.has_role(user, role, resource);

role(_user: User, "owner", _org: Org);
role(user: User, "owner", org: Org) if role(user, "member", org);
