allow("a","b","c");

a(a_var, x_val) if a_var = new A{x: x_val};
?= a(a_instance, "hello") and a_instance.x = "hello";

predicate(a: A, c: C) if
    a.x = c.y;
