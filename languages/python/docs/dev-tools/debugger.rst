########
Debugger
########

The Polar debugger allows debugging of policy rules. It can be helpful to see
why a rule is behaving differently than expected.

.. highlight:: polar

********************
Running the debugger
********************

The debugger can be entered through two mechanisms:

  1. Add the ``debug()`` predicate in the body of the rule you want to debug in
     your Polar file. The debugger will be entered when that rule body is
     evaluated by the Polar engine. For example::

       myRule() := debug(), 1 = 0;

  2. In the REPL, issue a query of the form: ``debug(), <query>``. This will
     enter the debugger before starting the query.

Entering the debugger will pause evaluation of the policy, and a command prompt will appear:

.. code-block:: terminal

  Welcome to the debugger!
  >

Since Polar is declarative, execution does not always flow sequentially through
a rule's body. Entering the debugger allows us to navigate through the
evaluation of a policy and interrogate the current state of the engine at every
step along the way.

************************
Available debug commands
************************

Help
====

``h[elp]``
----------

Print the debugger command reference.

.. code-block:: terminal

  > help
  Debugger Commands
    bindings                Print current binding stack.
    c[ontinue]              Continue evaluation.
    goals                   Print current goal stack.
    h[elp]                  Print this help documentation.
    l[ine] [<n>]            Print the current line and <n> lines of context.
    n[ext]                  Alias for 'over'.
    out                     Stop at the next rule.
    over                    Stop at the next query.
    queries                 Print current query stack.
    q[uit]                  Alias for 'continue'.
    stack                   Alias for 'queries'.
    s[tep]                  Evaluate one goal.
    var [<name> ...]        Print available variables. If one or more arguments
                            are provided, print the value of those variables.

Navigation
==========

``s[tep]``
----------

Evaluate one goal. This is *very* low level.

.. code-block:: terminal

  > line
  003: c() := debug("c");
              ^
  > step
  PopQuery(debug("c"))
  > line
  001: a() := debug("a"), b(), c(), d();
                               ^

``c[ontinue]`` or ``q[uit]``
----------------------------

Continue evaluation.

.. code-block:: terminal

  > line
  001: a() := debug("a"), b(), c(), d();
                               ^
  > continue
  [exit]

``over`` or ``n[ext]``
----------------------

Continue evaluation until the next query.

.. code-block:: terminal

  Welcome to the debugger!
  > line
  001: a() := debug(), b(), c(), d();
              ^
  > over
  001: a() := debug(), b(), c(), d();
                       ^
  > over
  001: a() := debug(), b(), c(), d();
                            ^
  > over
  Welcome to the debugger!
  > line
  003: c() := debug();
              ^
  > over
  001: a() := debug(), b(), c(), d();
                                 ^
  > over
  [exit]

``out``
-------

Continue evaluation until the next rule.

.. code-block:: terminal

  Welcome to the debugger!
  > line
  001: a() := debug(), b(), c(), d();
              ^
  > out
  Welcome to the debugger!
  > line
  003: c() := debug();
              ^
  > out
  001: a() := debug(), b(), c(), d();
                                 ^
  > out
  [exit]

Context
=======

``goals``
---------

Print current stack of goals.

.. code-block:: terminal

  Welcome to the debugger!
  > line
  001: a() := debug(), b(), c(), d();
              ^
  > goals
  PopQuery(a)
  PopQuery(debug,b,c,d)
  Query(d)
  Query(c)
  Query(b)
  PopQuery(debug)

``l[ine] [<n>]``
----------------

For the current stop point, print the corresponding Polar line and ``<n>``
lines of additional context above and below it.

.. code-block:: terminal

  > line
  003: c() := debug("c");
              ^
  > line 2
  001: a() := debug("a"), b(), c(), d();
  002: b();
  003: c() := debug("c");
              ^
  004: d();

``queries`` or ``stack``
------------------------

Print current stack of queries.

.. code-block:: terminal

  > line
  001: a() := debug(), b(), c(), d();
              ^
  > queries
  a()
  debug(), b(), c(), d()
  debug()

Variables
=========

``var [<var> ...]``
-----------------

Print variables in the current scope. If one or more arguments are provided, print the value of those variables. If a provided variable does not exist in the current scope, print ``<unbound>``.

.. note:: Due to temporaries used inside the engine, variables may not be
          available under the names used in the Polar file. ``var`` with no
          argument will list variable names in the current scope.

.. code-block:: terminal

  > line
  001: a() := z = y, y = x, x = 3, debug();
  > var
  _z_23, _y_24, _x_25
  > var _z_23 _x_25
  _z_23 = 3
  _x_25 = 3
  > var foo
  foo = <unbound>

``bindings``
------------

Print all variable bindings in the current scope.

.. code-block:: terminal

  > line
  001: a() := x = y, y = z, z = 3, debug();
                                   ^
  > bindings
  _x_32 = _y_33
  _y_33 = _z_34
  _z_34 = 3
