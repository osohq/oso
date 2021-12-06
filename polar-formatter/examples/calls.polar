allow(x) if
  is_active(x);

allow(x) if
  is_active(x, y, z);

allow(x) if
  is_okay(x, kwarg: "test", really_long_set_of_annoying_parameters: "some_really_long_string");
