allow(x) if
  is_active(x);

allow(x) if
  is_active(x, y, z);

allow(x) if
  x.long_method_name(really_long_set_of_annoying_parameters: "some_really_long_string");

allow(x) if
  is_okay(x, kwarg: "test", really_long_set_of_annoying_parameters: "some_really_long_string");

allow(x) if
  some_longer_call_name(x, really_long_set_of_annoying_parameters, kwarg: "test");

allow(x) if
  a_dot_operator.some_longer_call_name(x, really_long_set_of_annoying_parameters, kwarg: "test")
  .another_call_that_should_be_on_a_different_line();

allow(x) if
  a_dot_operator.some_longer_call_name(x, really_long_set_of_annoying_parameters, kwarg: "test")
  .another_call_that_should_be_on_a_different_line()
  .and_one_more();
