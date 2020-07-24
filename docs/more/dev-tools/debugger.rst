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
     evaluated by the Polar VM. For example::

       myRule() if debug() and 1 = 0;

  2. In the REPL, issue a query of the form: ``debug() and <query>``. This will
     enter the debugger before starting the query.

Entering the debugger will pause evaluation of the policy, and a command prompt
will appear:

.. code-block:: console

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

.. code-block:: console

  > help
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

  > line
  003: c() if debug();
              ^
  > step
  PopQuery(debug)
  > step
  PopQuery(debug)
  > line
  001: a() if debug() and b() and c() and d();
                                  ^

``c[ontinue]`` or ``q[uit]``
----------------------------

Continue evaluation.

.. code-block:: console

  > line
  001: a() if debug() and b() and c() and d();
                                  ^
  > continue
  [exit]

``over`` or ``n[ext]``
----------------------

Continue evaluation until the next query.

.. code-block:: console

  Welcome to the debugger!
  > line
  001: a() if debug() and b() and c() and d();
              ^
  > over
  001: a() if debug() and b() and c() and d();
                          ^
  > over
  001: a() if debug() and b() and c() and d();
                                  ^
  > over
  Welcome to the debugger!
  > line
  003: c() if debug();
              ^
  > over
  001: a() if debug() and b() and c() and d();
                                          ^
  > over
  [exit]

``out``
-------

Evaluate goals through the end of the current parent query and stop at the next
sibling of the parent query (if one exists).

.. code-block:: console

  Welcome to the debugger!
  > line
  001: a() if debug() and b() and c() and d();
              ^
  > out
  Welcome to the debugger!
  > line
  003: c() if debug();
              ^
  > out
  001: a() if debug() and b() and c() and d();
                                          ^
  > out
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
  > line
  001: a() if debug() and b() and c() and d();
              ^
  > goals
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

  > line
  003: c() if debug();
              ^
  > line 2
  001: a() if debug() and b() and c() and d();
  002: b();
  003: c() if debug();
              ^
  004: d();

``queries`` or ``stack``
------------------------

Print current stack of queries.

.. code-block:: console

  > line
  001: a() if debug() and b() and c() and d();
              ^
  > queries
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

  > line
  001: a() if x = y and y = z and z = 3 and debug();
                                   ^
  > var
  _y_22, _x_21, _z_23
  > var _x_21 _z_23
  _x_21 = 3
  _z_23 = 3
  > var foo
  foo = <unbound>


``bindings``
------------

Print all variable bindings in the current scope.

.. code-block:: console

  > line
  001: a() if x = y and y = z and z = 3 and debug();
                                            ^
  > bindings
  _x_21 = _y_22
  _y_22 = _z_23
  _z_23 = 3
