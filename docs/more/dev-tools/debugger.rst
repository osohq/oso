########
Debugger
########

The Polar debugger allows debugging of policy rules. It can be helpful to see
why a rule is behaving differently than expected.

********************
Running the debugger
********************

The debugger is entered through the ``debug()`` predicate. It acts like
a break point: when the VM tries to query for that predicate, it stops
instead and enters the debugger. You may put it anywhere in the body of
a rule:

.. code-block:: polar

     some_rule(x) if debug(x) and 1 = 0;

You can also query for it directly from the REPL:

.. code-block:: console

    query> debug()

When evaluation hits a ``debug()``, the ``debug>`` prompt will appear:

.. code-block:: console

  Welcome to the debugger!
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

.. code-block:: console

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
  b();
  c() if debug();
  d();

``s[tep]``
----------

Evaluate one goal (one instruction on the Polar VM). This is *very* low level.

.. code-block:: console

  debug> line
  003: c() if debug();
              ^
  debug> step
  PopQuery(debug)
  debug> step
  PopQuery(debug)
  debug> line
  001: a() if debug() and b() and c() and d();
                                  ^

``c[ontinue]`` or ``q[uit]``
----------------------------

Continue evaluation after the ``debug()`` predicate.

.. code-block:: console

  debug> line
  001: a() if debug() and b() and c() and d();
                                  ^
  debug> continue
  [exit]

``over`` or ``n[ext]``
----------------------

Continue evaluation until the next query.

.. code-block:: console

  Welcome to the debugger!
  debug> line
  001: a() if debug() and b() and c() and d();
              ^
  debug> over
  001: a() if debug() and b() and c() and d();
                          ^
  debug> over
  001: a() if debug() and b() and c() and d();
                                  ^
  debug> over
  Welcome to the debugger!
  debug> line
  003: c() if debug();
              ^
  debug> over
  001: a() if debug() and b() and c() and d();
                                          ^
  debug> over
  [exit]

``out``
-------

Evaluate goals through the end of the current parent query and stop at the next
sibling of the parent query (if one exists).

.. code-block:: console

  Welcome to the debugger!
  debug> line
  001: a() if debug() and b() and c() and d();
              ^
  debug> out
  Welcome to the debugger!
  debug> line
  003: c() if debug();
              ^
  debug> out
  001: a() if debug() and b() and c() and d();
                                          ^
  debug> out
  [exit]

Context
=======

The Polar file used in the following examples looks like this:

.. code-block:: polar

  a() if debug() and b() and c() and d();
  b();
  c() if debug();
  d();

``goals``
---------

Print current stack of goals.

.. code-block:: console

  Welcome to the debugger!
  debug> line
  001: a() if debug() and b() and c() and d();
              ^
  debug> goals
  PopQuery(a())
  PopQuery(debug(), b(), c(), d())
  Query(d())
  Query(c())
  Query(b())
  PopQuery(debug())

``l[ine] [<n>]``
----------------

For the current stop point, print the corresponding Polar line and ``<n>``
lines of additional context above and below it.

.. code-block:: console

  debug> line
  003: c() if debug();
              ^
  debug> line 2
  001: a() if debug() and b() and c() and d();
  002: b();
  003: c() if debug();
              ^
  004: d();

``queries`` or ``stack``
------------------------

Print current stack of queries.

.. code-block:: console

  debug> line
  001: a() if debug() and b() and c() and d();
              ^
  debug> queries
  a()
  debug() and b() and c() and d()
  debug()

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

.. code-block:: console

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

.. code-block:: console

  debug> line
  001: a() if x = y and y = z and z = 3 and debug();
                                            ^
  debug> bindings
  _x_21 = _y_22
  _y_22 = _z_23
  _z_23 = 3
