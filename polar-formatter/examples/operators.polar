allow(x) if
  1 + 2 * 3 + 4 * (5 + 6) * (x and y) and z;

allow(x) if
  some_really_long_string + something_else and
  something_else_long_that_would_not_all_fit_on_one_line;

allow(x) if
  some_really_long_string + really_long_operand_variable_name_with_method_call.test_some_ridiculously_long_method_name() and
  something_else;

allow(x) if
  (x + y).is_true();

allow(x) if
  forall(y in x, y < 10);

allow(x) if
  print(x);

allow(x) if
  debug();

allow(x) if
  cut;

allow(x) if
  new Test(x, y, z);
