########
Debugger
########

The Polar debugger allows debugging of policy rules. It can be helpful to see
why a rule is behaving differently than expected.

********************
Running the Debugger
********************

The debugger is entered through the ``debug()`` predicate. It acts like
a break point: when the VM tries to query for that predicate, it stops
instead and enters the debugger. You may put it anywhere in the body of
a rule:

.. code-block:: polar

     some_rule(x) if debug(x) and 1 = 0;

You can also query for it directly from the REPL:

.. code-block:: oso

    query> debug()

When evaluation hits a ``debug()``, the ``debug>`` prompt will appear:

.. code-block:: oso

  debug>

The debugger operates as a simple command-driven REPL, much like other
low-level debuggers such as GDB, LLDB, or JDB. You can exit the debugger
at any time by typing ``continue`` or ``quit`` followed by ``Enter``,
or by typing ``Ctrl-D`` (EOF).

*****************
Debugger Commands
*****************

Help
====

``h[elp]``
----------

Print the debugger command reference.

.. code-block:: oso

  debug> help
  Debugger Commands
  bindings                Print current binding stack.
  c[ontinue]              Continue evaluation.
  goals                   Print current goal stack.
  h[elp]                  Print this help documentation.
  l[ine] [<n>]            Print the current line and <n> lines of context.
  n[ext]                  Alias for 'over'.
  out                     Evaluate goals through the end of the current parent
                          query and stop at its next sibling (if one exists).
  over                    Evaluate goals until reaching the next sibling of the
                          current query (if one exists).
  queries                 Print current query stack.
  q[uit]                  Alias for 'continue'.
  stack                   Alias for 'queries'.
  s[tep]                  Evaluate one goal.
  var [<name> ...]        Print available variables. If one or more arguments
                          are provided, print the value of those variables.

Navigation
==========

The debugger allows you to navigate through the evaluation of a policy and
interrogate the current state of the engine at every step along the way.

The Polar file used in the following examples looks like this:

.. code-block:: polar

  a() if debug() and b() and c() and d();
  a() if 5 = 5;
  b() if 1 = 1 and 2 = 2;
  c() if 3 = 3 and 4 = 4;
  d();

``c[ontinue]`` or ``q[uit]``
----------------------------

Continue evaluation after the ``debug()`` predicate.

.. code-block:: oso

  debug> line
  001: a() if debug() and b() and c() and d();
              ^
  debug> continue
  [exit]


``s[tep]`` or ``into``
----------------------
Step to the next query. This is the lowest-level step of Polar's logical evaluation process.
After each step, the debugger prints the currently query, relevant bindings, and context from the policy file.

.. code-block:: oso

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


``over`` or ``n[ext]``
----------------------

Step to the next query at the same level of the query stack. This command is the same as ``step``, but it will not enter a lower
level of the stack. For example, it will not step into the body of a rule.

.. code-block:: oso

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

``out``
-------

Step out of the current level of the query stack, and stop at the next query at the level above.
Can be thought of as stepping to the next sibling of the current parent query (if one exists).

.. code-block:: oso

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

Context
=======

The Polar file used in the following examples looks like this:

.. code-block:: polar

  a(x) if debug() and b(x) and c();
  b(x) if (y = 1 and x = y) and y = 1;
  c() if 3 = 3 and 4 = 4;


``l[ine] [<n>]``
----------------

For the current stop point, print the corresponding Polar line and ``<n>``
lines of additional context above and below it.

.. code-block:: oso

  debug> line
  001: a(x) if debug() and b(x) and c();
                           ^

  debug> line 2
  001: a(x) if debug() and b(x) and c();
                           ^
  002: b(x) if (y = 1 and x = y) and y = 1;
  003: c() if 3 = 3 and 4 = 4;

``stack`` or ``trace``
------------------------

Print current stack of queries.

.. code-block:: oso

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


``query [<i>]``
---------------

Print the current query (no arguments), or the query at level ``i`` of the query stack.

.. code-block:: oso

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

Variables
=========

The Polar file used in the following examples looks like this:

.. code-block:: polar

  a() if x = y and y = z and z = 3 and debug();

``var [<var> ...]``
-------------------

Print variables in the current scope. If one or more arguments are provided,
print the value of those variables. If a provided variable does not exist in
the current scope, print ``<unbound>``.

.. note:: Due to temporaries used inside the engine, variables may not be
          available under the names used in the Polar file. ``var`` with no
          argument will list variable names in the current scope.

.. code-block:: oso

  debug> line
  001: a() if x = y and y = z and z = 3 and debug();
                                   ^
  debug> var
  _y_22, _x_21, _z_23
  debug> var _x_21 _z_23
  _x_21 = 3
  _z_23 = 3
  debug> var foo
  foo = <unbound>


``bindings``
------------

Print all variable bindings in the current scope.

.. code-block:: oso

  debug> line
  001: a() if x = y and y = z and z = 3 and debug();
                                            ^
  debug> bindings
  _x_21 = _y_22
  _y_22 = _z_23
  _z_23 = 3
