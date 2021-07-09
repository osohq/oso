allow("sam", "get", resource: Resource) if
    fib(12, res) and
    resource.id = res;

fib(1, 1);
fib(0, 1);
fib(n, res) if
    fib(n-1, x) and fib(n-2, y) and
    res = x + y; 