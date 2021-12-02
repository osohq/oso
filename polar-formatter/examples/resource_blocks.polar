resource Organization {
  permissions = ["one", "two", "three"];
  roles = ["hello", "world"];

  "one" if "hello";
  "two" if "one";

  "hello" if "world";
}

has_role(user: User, role: String, org: Organization) if
  {role, organization} in user.roles;
