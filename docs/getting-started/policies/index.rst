================
Writing Policies
================

If this is your first experience with writing declarative policies, welcome!
As a reminder: oso policies are written in a language called Polar.
There is a full :doc:`/using/polar-syntax` guide which you can use as a reference
of all the available syntax, but here we'll give an overview of
getting started with writing policies.

The syntax might feel a bit foreign at first, but fear not: almost
anything you can express in imperative code can equally be expressed in Polar
--- often more concisely and closer to how you might explain the logic in
natural language.


Matching
========

One of the core concepts to understand when writing oso policies, is that it
is all based around matching.

Take the basic ``allow`` rule that we use as a convention for where policy
decision start:

.. code-block:: polar

    allow(actor, action, resource) if ...

When we use ``oso.allow``, we are making a ``Polar`` query, and asking it to
find all rules that match (a) on the rule name "allow", and (b) on all the inputs.

In the above, ``actor``, ``action``, and ``resource`` were all simple parameter names.
I.e. these are new variables. These will match *anything*.

But we can replace one for a concrete type:

.. code-block:: polar

    allow(actor, "read", resource) if ...

Which is instead making sure the second input will match exactly with the string ``"read"``.

.. _if_statement:

``if`` statements
=================

There are several ways to represent imperative ``if`` logic in Polar.

In a rule body
^^^^^^^^^^^^^^

The most obvious way to write an ``if`` statement in Polar is to add a body to
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
approve any expense report by adding a :ref:`specializer <inheritance>` to the
rule head:

.. code-block:: polar

  allow(_actor: Admin, "approve", _report);

The rule will fail when evaluated on a regular ``User`` and succeed when
evaluated on an ``Admin``, encoding an implicit ``if Admin`` condition.

This is another example of the matching process: instead of matching against
a concrete value, we are instead checking to make sure the type of the input
matches the expected type - in this case, an ``Admin``.

.. tip::

    Try to use type specializers as often as possible. It will help make sure
    you don't accidentally allow access to an unrelated resource which happens
    to have matching fields.

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

.. tip:: 

  Interested in understanding more about leveraging application types for
  specializers and attribute lookups? Continue on to :doc:`application-types`.

  To see more policy examples, we have guides oriented around specific models
  like RBAC and ABAC in the :doc:`/using/examples/index` section.

  And finally, if you want a reference of all possible syntax,there is the
  :doc:`/using/polar-syntax` page.


.. toctree::
    :hidden:

    Guide <self>
    application-types
