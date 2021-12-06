allow(x) if
  is_active(x);

allow(x) if
  is_active(x, y, z);

allow(x) if
  is_okay(x, kwarg: "test", really_long_set_of_annoying_parameters: "some_really_long_string");

allow(x) if
  some_longer_call_name(x, really_long_set_of_annoying_parameters, kwarg: "test");
