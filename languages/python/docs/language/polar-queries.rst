=====================
Queries & unification
=====================

.. note:: This guide assumes you have a basic familiarity with the :doc:`polar-syntax`.

Now that we have described the basics of syntax and facts, let's consider how
queries behave.  Queries are used to ask questions of the knowledge base.

Application integrations will issue standardized queries to determine whether
requests are authorized.  For now, we will consider the basics of querying.

Let's consider the following example.  You can follow along by creating a new
file called ``learn-queries.polar``.  Polar includes a REPL that can be used to
issue queries.  From an environment that has the polar module installed,
run::

  $ python -m polar.parser --interactive learn-queries.polar

Queries of facts
================

.. highlight:: polar

Let's add a few basic facts to our file::

  person("sam", scott);
  person("david", "hatch");

  company("oso");

  employee(company("oso"), person("sam", "scott"));
  employee(company("oso"), person("david", "hatch"));

We can use queries in the REPL to check what information has been added to the
knowledge base::

  ?= person("sam", "scott");
  True

``True`` indicates that this fact exists in the knowledge base. If we try
another query, for example ``?= person("graham", "neray")``, the result will be
``False``.

Queries with variables
======================

Of course, these static queries are not extremely powerful. Instead, we can
include variables in a query.  When we do this, Polar will return all possible
values of the variable(s) that make the query True.

For example::

  person(first, last);

Will return to us::

  first = "sam"
  last = "scott"

  first = "david"
  last = "hatch"

This indicates sets of bindings that would make the query true.

The process of finding these bindings is called **unification**. Let's see more
generally how unification works with a few examples::

  ?= 0 = 1
  False

  ?= 1 = 1
  True

Recall that ``=`` is the unification operator.  If the operands are values, they
will unify if they are equal.  This holds for compound types like tuples as
well::

  ?= ("a", "b") = ("a", "b")
  True

  ?= ("a", "b") = ("b", "b")
  False

When we introduce variables, the unification operator will do one of two possible
actions:

  1. If the variable is not already bound, bind it to the other operand.
  2. If the variable is bound, check if it is equal to the other operand.

::

  ?= x = 1
  x = 1

  ?= 1 = x
  x = 1

  ?= x = 1, y = 2, x = y
  False

This final example is false because ``x`` and ``y`` are first bound to
values that are not equal.

Variable unification can be used to extract values from compound types::

  ?= person("sam", "scott") = person("sam", last)
  last = "scott"

When we do not use the unification operator directly in queries, Polar is
attempting to unify our query with every fact in the knowledge base. If the
query can be unified, the values of variables in that query are output as a set
of bindings.

Queries with rules
==================

Rules help us express logic conditions about facts, and abstractions for
querying.  Consider the query::

  ?= employee(company("oso"), employee)

Based on the facts in our knowledge base, this will return all employees of
oso::

  employee = person("sam", "scott")
  employee = person("david", "hatch")

Let's say we wanted to abstract this query slightly.  Add the following rule to
``learn-queries.polar``::

  oso_employee(employee) := employee(company("oso"), employee);

The query ``?= oso_employee(employee)`` now returns::

  employee = person("sam", "scott")
  employee = person("david", "hatch")

During evaluation, Polar first tries to unify the query with facts in the
knowledge base, or heads of rules (recall that the *head* is the part before
``:=``).  If the head unifies, Polar attempts to unify the body with the
knowledge base. During unification of the head, variables are bound.  If they
are referenced again in the body, they take the value obtained during
unification of the head.

We could instead query for ``?= oso_employee(person("david", "hatch"))``, which would
return ``True`` or ``False`` depending on the contents of the knowledge base
(since there are no variables in the query).

