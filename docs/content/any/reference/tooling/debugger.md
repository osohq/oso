---
title: Debugger
aliases:
    - ../more/dev-tools/debugger.html
description: |
  Use the Polar debugger to debug policy files.
---

# Debugger

The Polar debugger allows debugging of policy rules. It can be helpful to see
why a rule is behaving differently than expected.

## Running the Debugger

The debugger is entered through the `debug()` predicate. It acts like
a break point: when the VM tries to query for that predicate, it stops
instead and enters the debugger. You may put it anywhere in the body of
a rule:

```
some_rule(x) if debug(x) and 1 = 0;
```

You can also query for it directly from the REPL:

```
query> debug()
```

When evaluation hits a `debug()`, the `debug>` prompt will appear:

```
debug>
```

The debugger operates as a simple command-driven REPL, much like other
low-level debuggers such as GDB, LLDB, or JDB. You can exit the debugger
at any time by typing `continue` or `quit` followed by `Enter`,
or by typing `Ctrl-D` (EOF).

## Debugger Commands

### Help

#### `h[elp]`

Print the debugger command reference.

```
debug> help
Debugger Commands
h[elp]                  Print this help documentation.
c[ontinue]              Continue evaluation.
s[tep] | into           Step to the next query (will step into rules).
n[ext] | over           Step to the next query at the same level of the query stack (will not step into rules).
o[ut]                   Step out of the current query stack level to the next query in the level above.
g[oal]                  Step to the next goal of the Polar VM.
e[rror]                 Step to the next error.
r[ule]                  Step to the next rule.
l[ine] [<n>]            Print the current line and <n> lines of context.
query [<i>]             Print the current query or the query at level <i> in the query stack.
stack | trace           Print the current query stack.
goals                   Print the current goal stack.
bindings                Print all bindings
var [<name> ...]        Print available variables. If one or more arguments
                        are provided, print the value of those variables.
q[uit]                  Alias for 'continue'.
```

### Navigation

The debugger allows you to navigate through the evaluation of a policy and
interrogate the current state of the engine at every step along the way.

The Polar file used in the following examples looks like this:

```
a() if debug() and b() and c() and d();
a() if 5 = 5;
b() if 1 = 1 and 2 = 2;
c() if 3 = 3 and 4 = 4;
d();
```

#### `c[ontinue]` or `q[uit]`

Continue evaluation after the `debug()` predicate.

```
debug> line
001: a() if debug() and b() and c() and d();
            ^
debug> continue
[exit]
```

#### `s[tep]` or `into`

Step to the next query. This is the lowest-level step of Polar’s logical evaluation process.
After each step, the debugger prints the current query, relevant bindings, and context from the policy file.

```
debug> line
001: a() if debug() and b() and c() and d();
            ^
debug> step
QUERY: b(), BINDINGS: {}

001: a() if debug() and b() and c() and d();
                        ^
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;

debug> step
QUERY: 1 = 1 and 2 = 2, BINDINGS: {}

001: a() if debug() and b() and c() and d();
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
            ^
004: c() if 3 = 3 and 4 = 4;
005: d();

debug> step
QUERY: 1 = 1, BINDINGS: {}

001: a() if debug() and b() and c() and d();
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
            ^
004: c() if 3 = 3 and 4 = 4;
005: d();

debug> step
QUERY: 2 = 2, BINDINGS: {}

001: a() if debug() and b() and c() and d();
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
                    ^
004: c() if 3 = 3 and 4 = 4;
005: d();
```

#### `over` or `n[ext]`

Step to the next query at the same level of the query stack. This command is the same as `step`, but it will not enter a lower
level of the stack. For example, it will not step into the body of a rule.

```
debug> line
001: a() if debug() and b() and c() and d();
            ^

debug> next
QUERY: b(), BINDINGS: {}

001: a() if debug() and b() and c() and d();
                        ^
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;

debug> next
QUERY: c(), BINDINGS: {}

001: a() if debug() and b() and c() and d();
                                ^
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;

debug> next
QUERY: d(), BINDINGS: {}

001: a() if debug() and b() and c() and d();
                                        ^
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;

debug> next
True
QUERY: 5 = 5, BINDINGS: {}

001: a() if debug() and b() and c() and d();
002: a() if 5 = 5;
            ^
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;
005: d();

debug> next
True
```

#### `out`

Step out of the current level of the query stack, and stop at the next query at the level above.
Can be thought of as stepping to the next sibling of the current parent query (if one exists).

```
debug> line
003: b() if 1 = 1 and 2 = 2;
            ^

debug> out
QUERY: c(), BINDINGS: {}

001: a() if debug() and b() and c() and d();
                                ^
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;

debug> step
QUERY: 3 = 3 and 4 = 4, BINDINGS: {}

001: a() if debug() and b() and c() and d();
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;
            ^
005: d();

debug> out
QUERY: d(), BINDINGS: {}

001: a() if debug() and b() and c() and d();
                                        ^
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;

debug> out
True
True
```

#### `g[oal]`

Step the the next goal of the Polar VM. A "goal" is an internal object used to direct
the state of the Polar interpreter, so this is mainly useful for debugging the VM itself.

```
query> debug() and 1 + 2 = 3
QUERY: debug(), BINDINGS: {}

001: debug() and 1 + 2 = 3
     ^

debug> goal
PopQuery(debug())
debug> g
Query(1 + 2 = _op_5 and _op_5 = 3)
debug> g
TraceStackPush
debug> c
True
```

#### `e[rror]`

Continue execution until an error condition is detected. The debugger will
then pause the Polar VM at that point, allowing the stack and bindings to
be examined.


```
query> x = 1 and y = "2" and debug() and x < y
QUERY: debug(), BINDINGS: {}

001: x = 1 and y = "2" and debug() and x < y
                           ^

debug> error
QUERY: 1 < "2", BINDINGS: {}

001: x = 1 and y = "2" and debug() and x < y
                                       ^

ERROR: Not supported: 1 < "2"

debug> stack
1: x = 1 and y = "2" and debug() and x < y
  in query at line 1, column 1
0: x < y
  in query at line 1, column 35

debug> var x y
x = 1
y = "2"
debug> c
UnsupportedError
Not supported: 1 < "2"
```

#### `r[ule]`

Step to the next rule invocation. This can be used to debug rule matching order.


```
QUERY: debug(), BINDINGS: {}

001: a() if debug() and b() and c() and d();
            ^
002: a() if 5 = 5;
003: b() if 1 = 1 and 2 = 2;
004: c() if 3 = 3 and 4 = 4;

debug> line
001: a() if debug() and b() and c() and d();
            ^
debug> rule
b() if 1 = 1 and 2 = 2;
debug> rule
c() if 3 = 3 and 4 = 4;
debug> rule
d();
debug> rule
True
a() if 5 = 5;
debug> rule
True
```

### Context

The Polar file used in the following examples looks like this:

```
a(x) if debug() and b(x) and c();
b(x) if (y = 1 and x = y) and y = 1;
c() if 3 = 3 and 4 = 4;
```

#### `l[ine] [<n>]`

For the current stop point, print the corresponding Polar line and `<n>`
lines of additional context above and below it.

```
debug> line
001: a(x) if debug() and b(x) and c();
                         ^

debug> line 2
001: a(x) if debug() and b(x) and c();
                         ^
002: b(x) if (y = 1 and x = y) and y = 1;
003: c() if 3 = 3 and 4 = 4;
```

#### `stack` or `trace`

Print current stack of queries.

```
debug> line
001: a(x) if debug() and b(x) and c();
                         ^
debug> stack
2: a(1)
  in query at line 1, column 1
1: debug() and b(x) and c()
  in rule a at line 1, column 9 in file test.polar
0: b(x)
  in rule a at line 1, column 21 in file test.polar

debug> step
QUERY: _y_6 = 1 and _x_5 = _y_6 and _y_6 = 1, BINDINGS: {_x_5 = 1}

001: a(x) if debug() and b(x) and c();
002: b(x) if (y = 1 and x = y) and y = 1;
             ^
003: c() if 3 = 3 and 4 = 4;

debug> stack
3: a(1)
  in query at line 1, column 1
2: debug() and b(x) and c()
  in rule a at line 1, column 9 in file test.polar
1: b(x)
  in rule a at line 1, column 21 in file test.polar
0: (y = 1 and x = y) and y = 1
  in rule b at line 2, column 9 in file test.polar

debug> out
QUERY: c(), BINDINGS: {}

001: a(x) if debug() and b(x) and c();
                                  ^
002: b(x) if (y = 1 and x = y) and y = 1;
003: c() if 3 = 3 and 4 = 4;

debug> stack
2: a(1)
  in query at line 1, column 1
1: debug() and b(x) and c()
  in rule a at line 1, column 9 in file test.polar
0: c()
  in rule a at line 1, column 30 in file test.polar
```

#### `query [<i>]`

Print the current query (no arguments), or the query at level `i` of the query stack.

```
debug> stack
4: a(1)
  in query at line 1, column 1
3: debug() and b(x) and c()
  in rule a at line 1, column 9 in file test.polar
2: b(x)
  in rule a at line 1, column 21 in file test.polar
1: (y = 1 and x = y) and y = 1
  in rule b at line 2, column 9 in file test.polar
0: y = 1 and x = y
  in rule b at line 2, column 10 in file test.polar

debug> query
QUERY: _y_12 = 1 and _x_11 = _y_12, BINDINGS: {_x_11 = 1}

debug> query 1
QUERY: _y_12 = 1 and _x_11 = _y_12 and _y_12 = 1, BINDINGS: {_x_11 = 1}

debug> query 2
QUERY: b(_x_8), BINDINGS: {_x_8 = 1}
```

### Variables

The Polar file used in the following examples looks like this:

```
a() if _x = y and y = z and z = 3 and debug();
```

#### `var [<var> ...]`

Print variables in the current scope. If one or more arguments are provided,
print the value of those variables. If a provided variable does not exist in
the current scope, print `<unbound>`.

{{% callout "Note" "green" %}}
  Due to temporaries used inside the engine, variables may not be bound
  under the exact names used in the Polar file. The debugger will automatically
  map supplied variable names to their temporary equivalents.
{{% /callout %}}

```
debug> line
001: a() if _x = y and y = z and z = 3 and debug();
                                 ^
debug> var
_y_2, __x_1, _z_3
debug> var _x z
_x@__x_1 = 3
z@_z_3 = 3
debug> var foo
foo = <unbound>
```

#### `bindings`

Print all variable bindings in the current scope.

```
debug> line
001: a() if x = y and y = z and z = 3 and debug();
                                          ^
debug> bindings
_x_21 = _y_22
_y_22 = _z_23
_z_23 = 3
```
