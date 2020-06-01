========
Debugger
========

The Polar debugger allows debugging of policy rules.  It can be helpful to see
why a rule is behaving differently than expected.

.. highlight:: polar

Running the debugger
--------------------

The debugger can be entered through several mechanisms.

1. From the oso API, call :py:meth:`oso.Oso.allow` with the argument
   ``debug=True``.
2. In the REPL, issue a query of the form: ``debug(), <query>``.  This will
   enter the debugger before starting the query.
3. Add the ``debug()`` predicate in the body of the rule you want to debug in
   your Polar file. This will cause the debugger to be entered when that rule
   body is evaluated by the Polar engine. The ``debug()`` predicate returns true
   in a rule evaluation, so it should normally be followed by an AND operator
   (``,``) and the rest of the rule body.

   For example::

     debugMe() := debug(), 1 = 0;

Currently the debugger will break in the current process and enter a debugging
command line prompt.

Debugger stop points
--------------------

Since Polar is declarative, execution does not always flow sequentially through
a rule's body. The polar debugger will stop at the following points:

- *call*: A rule is called and will be evaluated in a scope of variable bindings.
- *retry*: Evaluation of a rule has completed and is being retried with a new
  set of possible variable bindings.

Available debug commands
------------------------

Navigation
^^^^^^^^^^

- ``step``: Go to the next stop point
- ``continue``: Go to the next break point (set by a ``debug()`` predicate).
- ``over``: Step over the current clause, skipping over any rules
  that are checked to evaluate it.
- ``out``: Step out of the current rule evaluation, stopping at the next clause
  one level up in the stack.

Context
^^^^^^^

- ``list``: Show Polar file line number and context for the current stop point.
- ``stack``: Print Polar stack showing rules that led to stopping at the current
  point.
- ``trace``: Print the Polar trace, showing clauses that evaluated to true to
  lead to the current stop point.

Variables
^^^^^^^^^

- ``var <var>``: Print the value of variable <var>.
  .. note:: Due to temporaries used inside the engine, variables may not be
  available under the names used in the Polar file.  ``var`` with no argument
  will list variable names in the current scope.
- ``bindings``: Print all variables that are assigned and their values.


Advanced
^^^^^^^^

- ``pdb``: Drop into a PDB debugging session at the current stop point.
