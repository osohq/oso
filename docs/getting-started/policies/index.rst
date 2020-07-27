================
Writing Policies
================

Policies are the source of truth for the authorization logic used to evaluate queries in oso.
As a reminder: oso policies are written in a declarative language called Polar.
There is a full :doc:`/using/polar-syntax` guide which you can use as a reference
of all the available syntax, but here we'll give an overview of
getting started with writing policies.

The syntax might feel a bit foreign at first, but fear not: almost
anything you can express in imperative code can equally be expressed in Polar
--- often more concisely and closer to how you might explain the logic in
natural language.

.. note::
    Policies are stored in Polar files (extension ``.polar``), which are loaded
    into the authorization engine using the oso :doc:`/using/libraries/index`.

Rule Basics
===========

Policies are made up of :ref:`rules <polar-rules>`. Each rule defines
a statement that is either `true` or `false`. oso answers queries by evaluating rules that match the
query name and parameters. Let's take a basic :ref:`allow rule<allow-rules>` as an example:

.. code-block:: polar

    allow(actor, action, resource) if ...


When we use :py:meth:`~oso.Oso.is_allowed()` (or equivalent), we are making a query that asks oso to
evaluate all rules that match *(a)* on the rule name ``"allow"``, and *(b)* on all the inputs.

In the rule above, ``actor``, ``action``, and ``resource`` are simply the parameter names,
i.e. they are variables that will match *anything*.


But if we replace ``action`` with a concrete type:

.. code-block:: polar

    allow(actor, "read", resource);

the rule will now only be evaluated if the second input exactly matches the string ``"read"``.

.. _if_statement:

``if`` statements
=================

There are several ways to represent imperative ``if`` logic in Polar.

In a rule body
^^^^^^^^^^^^^^

The most common way to write an ``if`` statement in Polar is to add a body to
a rule. The following rule allows **any** actor to approve **any** expense report:

.. code-block:: polar

  allow(_actor, "approve", _report);

To restrict the rule such that only administrators may approve any expense
report, we can add a body:

.. code-block:: polar

  allow(actor, "approve", _report) if
      actor.is_admin = true;

To express multiple truth conditions (e.g., ``if A or B, then...``), we can
either create multiple rules...

.. code-block:: polar
  :emphasize-lines: 4-5

  allow(actor, "approve", _report) if
      actor.is_admin = true;

  allow(actor, "approve", _report) if
      actor.title = "CFO";

...or we can use Polar's :ref:`disjunction` operator (OR) to combine the conditions
in a single rule body:

.. code-block:: polar
  :emphasize-lines: 3

  allow(actor, "approve", _report) if
      actor.is_admin = true
      or actor.title = "CFO";

.. tip::

    In these rules we declared some variables with leading understores
    (``_report``, ``_actor``).  A leading underscore indicates that the variable
    will only be used once (Polar does not distinguish between definition and
    use). These variables are called *singleton variables*, and will match any
    value.  To help prevent errors, a warning will be emitted if a singleton variable
    is not preceeded by an underscore.

As specializers in a rule head
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Given the following application class structure...

.. code-block:: python
  :caption: :fab:`python`

  class User:
      ...

  class Admin(User):
      ...

...we can modify our original bodiless rule to only allow ``Admin`` users to
approve any expense report by adding a :ref:`specializer <specializer>` to the
rule head:

.. code-block:: polar

  allow(_actor: Admin, "approve", _report);

The rule will fail when evaluated on a regular ``User`` and succeed when
evaluated on an ``Admin``, encoding an implicit ``if Admin`` condition.

This is another example of the rule matching process: instead of matching against
a concrete value, we are instead checking to make sure the type of the input
matches the expected type - in this case, an ``Admin``.

.. tip::

    Try to use type specializers as often as possible. It will help make sure
    you don't accidentally allow access to an unrelated resource which happens
    to have matching fields.

.. _combining_rules:

Combining Rules
===============

Rules can be thought of as equivalent to methods in imperative programming.
And the same idea should be applied when writing policies: any pieces
of logic that you want to reuse throughout a policy can be extracted out into
a new rule.

The benefit of this is (a) it makes it easier to keep logic consistent throughout,
and (b) it often results in much more readable policy.

Take the following example. We want a rule saying that accountants
can read expenses.
Our initial version might look like:

.. code-block:: polar

    allow(user: User, "read", expense: Expense) if
      user.role = "accountant";

This would be fine, but if, for example, we wanted to allow the CFO to
do whatever an accountant can do, we would need to duplicate all the rules.
Or if we want to change how an application determines roles we would need
to change all locations using this.

So instead, we can refactor the role check into its own rule:

.. code-block:: polar

    allow(user: User, "read", expense: Expense) if
      role(user, "accountant");

    role(user, role_name) if user.role = role_name;


The ``role(user, "accountant")`` is yet another example of matching happening
in Polar. Any time a rule body contains a **predicate** like this, it is performing
another query. I.e. it will try and find all *matching* rules called "role" with
two inputs.

You can also either use the :doc:`/more/dev-tools/repl` or the ``oso.query_predicate``
method to interact with this directly. For example:

.. code-block:: python
  :caption: :fab:`python` user.py
  :class: copybutton

  from oso import Oso

  class User:
      def __init__(self, name, role):
          self.name = name
          self.role = role

  oso = Oso()
  oso.load_str("role(user, role_name) if user.role = role_name;")

  alice = User("alice", "accountant")
  assert oso.query_predicate("role", alice, "accountant")

Summary
=======

We covered some of the basics of policies, how to represent conditional
logic, and the core idea of matching.

.. admonition:: What's next
    :class: tip whats-next

    * Interested in understanding more about leveraging application types for
      specializers and attribute lookups? Continue on to :doc:`application-types`.
    * To see more policy examples---including guides oriented around specific models
      like role-based or attribute-based access control---see :doc:`/using/examples/index`.
    * To continue learning policy syntax, go on to :doc:`/using/polar-syntax`.


.. toctree::
    :hidden:

    Guide <self>
    application-types
