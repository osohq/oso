allow(user: User{test: true}, "update", document: Document) if
  document.owner = user and
  user.active = true;

allow(user: User{test: true, really_long_set_of_annoying_matching_parameters: "some_val"}, "update", document: Document) if
  document.owner = user and
  user.active = true;

has_role(user: User, role_name: String, document: Document) if
  role in user.roles and
  role matches {name: role_name, document};
