example_integer(1);
example_string("abc");
example_bool(true);
example_homogeneous_list([1, 2, 3]);
example_heteregenous_list([1, "abc", true]);

is_integer(_i: Integer);
is_string(_s: String);
is_bool(_b: Boolean);
is_list(_l: List);
is_dict(_d: Dictionary);
is_null(nil);