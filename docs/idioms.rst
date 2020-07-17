==========================
Imperative Idioms in Polar
==========================

If this is your first experience with logic programming, welcome! Logic
programming is a powerful paradigm that's well suited to writing authorization
logic. The syntax might feel a bit foreign at first, but fear not: almost
anything you can express in imperative code can equally be expressed in Polar
--- often more concisely and closer to how you might explain the logic in
natural language.

.. _if_statement:

``if`` statements
=================

There are several ways to represent imperative ``if`` logic in Polar.

In a rule body
^^^^^^^^^^^^^^

The most obvious way to write an ``if`` statement in Polar is to add a body to
a rule. The following rule allows any actor to approve any expense report:

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

...or we can use Polar's :ref:`disjunction` operator to combine the conditions
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

.. TODO: ``else`` with ``cut``?

.. _lists:

List operations
===============

Polar has first-class support for lists, and you can perform a variety of
common operations on lists.

Membership
^^^^^^^^^^

When writing authorization code, it's common to check for membership in an
explicit allow- or deny-list. Polar provides the :ref:`in <operator-in>` operator to
perform list membership checks:

.. code-block:: polar

  prime(n) if
      n in [2, 3, 5, 7, ...];
