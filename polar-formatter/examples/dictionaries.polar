test(x) if
  { really_long_variable: ReallyLongValue, nested: { nest_the_thing_up: true }, val: 5, role: "admin"};

test(
# comment
  x
) if
  { really_long_variable: ReallyLongValue, nested: { s: true }, val: 5, role: "admin"};

#comment
test(x: { really_long_variable: ReallyLongValue, nested: { s: true }, role: "admin"}) if true;
