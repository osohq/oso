============
Syntax Guide
============

Polar is a Prolog based logic programming language, specialized for making
authorization decisions and tightly integrating with your application's native
language.

This guide is a brief description of the core syntax elements of Polar.

Each Polar file defines a set of facts and rules.  When a Polar file is loaded
into the authorization engine, all facts and rules are added to the engine's knowledge base.
This knowledge base is similar to a specialized database.

The knowledge base may be queried.  The behavior of queries is described further
in :doc:`polar-queries`.

.. _basic-types:

Basic Types
===========

Polar has only a few basic data types.

Numbers
-------

Polar parses unquoted integers as numeric values. For example::

  22
  43
  7

are all parsed as numbers. Numbers only compare equal with other numbers of the
same value.

Strings
-------

Polar supports quoted strings, which can be used to represent any textual data.
Polar strings are quoted with double quotes (``"``). Quotes within strings can
be escaped with a single backslash. Two strings are considered equal if they
have the same length and each of their corresponding characters are equal.

.. _compound-types:

Compound Types
==============

To support more complex data, Polar includes the following compound data types.

Tuples
------

A tuple is a sequence of values, defined using parentheses ``(v1, v2, ...,
vn)``.

.. highlight:: polar

For example::

  ("sam", "scott")
  ("polar", "lang", "oso")
  ("oso", ("polar", "lang"))

Tuples may have any length. Two tuples are equal if they have the same
length and all of the corresponding elements are equal.

.. _dictionaries:

Dictionaries
------------

.. note::

  This is an area of active development! Syntax is likely to change and
  evolve.

While tuples are useful for representing ordered data, dictionaries
(aka hash tables or associative arrays) can be more expressive for
relational data such as mappings. Dictionaries are another core type
in Polar, represented as::

  {key1: value1, key2: value2, ..., keyN: valueN}

For example::

  {"first_name": "sam", "last_name": "scott"}

Classes
-------

A similar syntax can be used to represent instances of classes.  The class
name is specified before the dictionary::

  Person{"first_name": "sam", "last_name": "scott"}

Classes can be registered from your application to integrate with Polar.  See
:doc:`/application-library/index` for more information.

Facts
=====

Facts are data added directly to the knowledge base. Facts are defined
in a Polar file and terminated with a semicolon. Instances of any of the
data types above may be defined as facts, but the most important kind
of facts are predicates, which we'll discuss next.

.. _predicates:

Predicates
----------

A tuple combined with a name is known as a ``predicate``.  Predicates take
the form ``name(arg1, ..., argN)``.  As we will see, predicates are the most basic construction
in Polar for accessing data and expressing authorization logic.

Some sample predicates::

  person("sam", "scott");
  company("oso");

.. _polar-rules:

Rules
=====

Data types and predicates are useful for representing and querying
information in the knowledge base, but they do not allow us to express
conditional ("**if** this **then** that") statements. We can use
rules to do this.

A rule in Polar takes the form::

  HEAD := BODY;

where ``HEAD`` must be a *fact* and ``BODY`` any number of *terms*.
The meaning of a rule is that ``HEAD`` is true **if** each of the ``BODY``
terms is true. There may be multiple rules with the same head; each
``BODY`` will be tried in turn, and any or all may succeed. For more
on how rules are defined and applied see :doc:`polar-queries`.

The following is an example of a rule::

  user("sam", "scott") := person("sam", "scott");

This example says that Sam is a user **if** he is also defined
as a person.

Terms
-----

A *term* is either a fact or a combination of facts using :ref:`operators`.

.. _variables:

Variables
---------

The example rule above is static. More powerful rules can be
formed using variables.  In Polar, a variable does not need a separate
declaration; it is created the first time it is referenced. Variables can be
substituted for values in dictionaries, or items in a tuple or predicate.

The following are all variables::

  foo
  bar
  myvar

To make the above rule more useful, we could write::

  user(first, last) := person(first, last);

This rule says that **if** there is a person with some name,
**then** that person is also a user.

.. _operators:

Operators
---------

.. todo not really true... some operators can be used in other places.

Operators are used to combine terms in rule bodies.

Unification
^^^^^^^^^^^

Unification is the basic matching operation in Polar. Two values are
said to *unify* if they are equal or if there is a consistent set of
variable bindings that makes them equal. Unification is defined
recursively over compound types (e.g., tuples and dictionaries):
two compound values unify if all of their corresponding elements
unify.

Unification may be performed explicitly with the unification operator
(``=``), which is true if its two operands unify; e.g., ``1 = 1``,
``"a" = "a"``, or ``x = 1`` where the variable ``x`` is either
bound to ``1`` or unbound.

Unification is also used to determine if queries match rule ``HEAD`` s,
and if the ``BODY`` of rules match other facts in the knowledge base.
We will cover unification further in :doc:`polar-queries`.

.. todo add a little table with unification examples, esp. w/dictionaries.

Conjunction (and)
^^^^^^^^^^^^^^^^^

To say that two terms in a rule's body must **both** be true,
the comma operator (``,`` pronounced "and") can be used. For
example, the rule::

  oso_user(first, last) :=
    user(first, last),
    employee(company("oso"), person(first, last));

will be satisfied if the named person is a user **and** that
person is an employee of oso.

.. _disjunction:

Disjunction (or)
^^^^^^^^^^^^^^^^^

The pipe operator (``|``, pronounced "or") will be true if either
its left **or** its right operand is true. Disjunctions can always
be replaced by multiple rules with identical heads but different bodies
(the operands), but may help simplify writing rules with alternatives.

Dictionary key access
^^^^^^^^^^^^^^^^^^^^^

The dot ``.`` operator can be used to access the value associated with
a key in a dictionary. For example, the rule::

  first_name(dict, x) :=
    dict = Person{},
    x = dict.first_name;

will access the value of the field named ``"first_name"`` in ``dict``,
and unify it with ``x``.

.. _numerical-comparison:

Numerical Comparison
^^^^^^^^^^^^^^^^^^^^^

The typical numerical comparison operators can be used to compare values.
``< <= > >= == !=``

For example::

  age < 10

will compare the value of the variable age with 10 and unify if it's less than 10.

.. _cut-operator:

Cut
^^^

.. note::
  This is an area of active development!
  The ``cut()`` operator does not currently prevent
  backtracking across rules, only within them.

The *cut* operator, which in Polar is written as ``cut()``, commits
the query engine to the enclosing rule definition, and refuses to
consider any others. Any definitions that have already run are not
"un-run", though, or avoided by using cut; it just ensures that no
*others* will run. Such "other" rule definitions are often less
specific rules (see :doc:`polar-classes`), and the use of `cut()`
can be used, e.g., to override an ``allow`` method on a less-specific
class.

``cut()`` can appear anywhere in a rule body, but terms that
proceed it must succeed in order for it to be reached, so it
frequently appears at the end of the body: **if** so-and-so is true,
then **cut** out all other alternatives.  ``cut()`` should be
used sparingly.

In (List Membership)
^^^^^^^^^^^^^^^^^^^^

The ``in`` operator can be used to iterate over a list. If the second operand is a list, the first operand will
be unified with each element of the list. If the
second operand is not a list (or variable bound to a list),
the operation will fail.

For example::

    x in [1, 2, 3], x = 1

Will bind ``x`` to ``1``, ``2``, ``3``, in turn, and check that ``x = 1``
for each. This expression will only succeed for the first item (``1``).

The ``in`` operator generates *alternatives* for each element of the list.

For all
^^^^^^^

The ``forall`` predicate is often useful in conjunction with the ``in`` operator.
``forall(condition, action)`` checks that ``action`` succeeds for every alternative
produced by ``condition``.

For example::

    forall(x in [1, 2, 3], x = 1)

Would fail because ``x`` only unifies with ``1`` for the first element in the
list (the first alternative of condition).

::

    forall(x in [1, 1, 1], x = 1)

succeeds because the ``action`` holds for all values in the list.

``forall`` can also be used with application data to check all elements returned
by an application method.

::
    forall(role = user.roles(), role = "admin")


Any bindings made inside a ``forall`` (``role`` or ``x`` in the example above)
cannot be accessed after the ``forall`` predicate.
