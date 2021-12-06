resource Organization {
  permissions = ["one", "two", "three", "so_many_permissions", "goes", "2", "lines"];
  roles = ["hello", "world", "too", "many", "roles", "to", "fit", "on", "1", "line", "sad"];
  relations = {parent: Tenant};

  "one" if "hello";
  "two" if "one";

  # stuff
  "hello" if "world" on "another";
}

has_role(user: User, role: String, org: Organization) if
  {role, organization} in user.roles;
