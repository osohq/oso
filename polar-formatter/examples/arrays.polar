test(x) if
  [
    really_long_variable,
    { nest_the_thing_up: true },
    "some_really_long_string",
    *rest_var
  ];

test(
  # comment
  x
) if
  [x, y, z];

#comment
test(
  some_longer_argument_name: [
    really_long_variable, { nest: true }, "some_really_long_string"
  ]
) if
  true;

test(some_longer_argument_name: ["some_really_long_string"]) if true;

test(action) if
  action in
    [
      "read",
      "create_repos",
      "list_repos",
      # "create_role_assignments",
      "list_role_assignments",
      "update_role_assignments"
      # "delete_role_assignments",
    ];
