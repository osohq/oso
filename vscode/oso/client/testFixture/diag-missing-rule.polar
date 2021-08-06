f(1);
f(2);

g(_x) if f(3);

# this one shouldn't cause a warning
g() if f(2);
