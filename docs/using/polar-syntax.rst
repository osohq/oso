============
Polar Syntax
============

Polar is a declarative logic programming language, specialized for making
authorization decisions and tightly integrating with your application's native
language.

This guide is a brief description of the core syntax elements of Polar.

Each Polar file defines a set of rules.  When a Polar file is loaded into the
authorization engine, all rules are added to the engine's knowledge base.

The knowledge base may be queried.  The behavior of queries is described further
:ref:`here <search-procedure>`.

.. _basic-types:

Primitive Types
================

Polar has only a few primitive data types.

.. _numbers:

Numbers
-------

Polar parses unquoted integers or floating point numbers as numeric values.
For example::

  22
  43
  -7
  22.3
  -22.31
  2.0e9

are all parsed as numbers.

You can also perform basic arithmetic on numbers with the operators
``+``, ``-``, ``*``, and ``/``.

.. _booleans:

Boolean
-------
Polar parses the keywords ``true`` and ``false`` as boolean values.

.. _strings:

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

.. _lists:

Lists
------

A list is a sequence of values, defined using brackets ``[v1, v2, ...,
vn]``.

.. highlight:: polar

For example::

  ["polar", "lang", "oso"]
  ["oso", ["polar", "lang"]]

Lists may have any length. List membership can be determined using the :ref:`in operator <operator-in>`.

.. _dictionaries:

Dictionaries
------------

While lists are useful for representing ordered data, dictionaries
(aka hash tables or associative arrays) can be more expressive for
relational data such as mappings. Dictionaries are another core type
in Polar, represented as::

  {key1: value1, key2: value2, ..., keyN: valueN}

For example::

  {first_name: "Yogi", last_name: "Bear"}

Class Instances
---------------

A similar syntax can be used to represent instances of classes.  The class
name is specified before the dictionary::

  Person{first_name: "Yogi", last_name: "Bear"}

Classes can be registered with the oso library to integrate with Polar.  See
:doc:`/getting-started/policies/application-types` for more information.

A class instance literal must be used either with the :ref:`new operator <operator-new>` or
as a :ref:`pattern <pattern>`.

.. _polar-rules:

Rules
=====

Every statement in a Polar file is part of a rule.  Rules allow us to express
conditional ("**if** this **then** that") statements.

A rule in Polar takes the form::

  HEAD if BODY;

where ``HEAD`` must be a *fact* and ``BODY`` any number of *terms*.
The meaning of a rule is that ``HEAD`` is true **if** each of the ``BODY``
terms is true. If there are be multiple rules with the same head, each
``BODY`` will be tried in turn, and any or all may succeed. For more
on how rules are defined and applied see
:doc:`/more/language/polar-foundations`.

The following is an example of a rule::

  user("yogi", "bear") if person("yogi", "bear");

This example says that Sam is a user **if** he is also defined
as a person.

Terms
-----

A *term* is either a data type or a combination of facts using :ref:`operators`.

.. _variables:

Variables
---------

The example rule above is static. More powerful rules can be
formed using variables.  In Polar, a variable does not need a separate
declaration; it is created the first time it is referenced. Variables can be
substituted for values in dictionaries, or items in a list or rule call.

The following are all variables::

  foo
  bar
  myvar

To make the above rule more useful, we could write::

  user(first, last) if person(first, last);

This rule says that **if** there is a person with some name,
**then** that person is also a user.

.. _singletons:

Singletons
^^^^^^^^^^

If a variable occurs only once, then its value can't be used
for anything. Such variables are called *singletons*, and Polar
will warn you if they occur in a rule; e.g., if you try to load
the rule::

  user(first, last) if person("George", last);

Polar will say::

  Singleton variable first is unused or undefined
  001: user(first, last) if person("George", last);
            ^

The reason these warnings are important is that, as in this case,
they indicate potential logical errors. Here, the error is forgetting
to use the first name, and instead using a literal string in the
call to ``person``.

There are cases, however, where it *isn't* an error to have
a singleton variable. For example:

* As a parameter with a specializer: ``allow(_actor: Person{first_name: "George"}, ..);``
* As a parameter that is explicitly ignored: ``always_true(_);``

In such cases, you can suppress the singleton variable warning by
starting your variable's name with an ``_`` (underscore), e.g.,
``_actor`` in the first example above.

A variable named *just* ``_`` (as in the second example above) is called
an **anonymous** variable, and it is *always* a singleton (but will never
generate a warning). Each occurrence is translated into a fresh variable,
guaranteed not to match any other variable. You may therefore have as many
anonymous variables in a rule as you like, and each will be unique.
It's up to you whether to use an anonymous variable or a singleton with
a descriptive name.

.. _operators:

Operators
---------

.. todo::
   not really true... some operators can be used in other places.

Operators are used to combine terms in rule bodies into expressions.

Unification
^^^^^^^^^^^

Unification is the basic matching operation in Polar. Two values are
said to *unify* if they are equal or if there is a consistent set of
variable bindings that makes them equal. Unification is defined
recursively over compound types (e.g., lists and dictionaries):
two compound values unify if all of their corresponding elements
unify.

Unification may be performed explicitly with the unification operator
(``=``), which is true if its two operands unify; e.g., ``1 = 1``,
``"a" = "a"``, or ``x = 1`` where the variable ``x`` is either
bound to ``1`` or unbound.

Unification is also used to determine if queries match rule ``HEAD`` s,
and if the ``BODY`` of rules match other facts in the knowledge base.
We will cover unification further in :ref:`search-procedure`.

.. todo::
   add a little table with unification examples, esp. w/dictionaries.

Conjunction (and)
^^^^^^^^^^^^^^^^^

To say that two terms in a rule's body must **both** be true,
the and operator (``and``) can be used. For
example, the rule::

  oso_user(first, last) if
    user(first, last) and
    employee(company("oso"), person(first, last));

will be satisfied if the named person is a user **and** that
person is an employee of oso.

.. _disjunction:

Disjunction (or)
^^^^^^^^^^^^^^^^^

The or operator (``or``) will be true if either
its left **or** its right operand is true. Disjunctions can always
be replaced by multiple rules with identical heads but different bodies
(the operands), but may help simplify writing rules with alternatives.

Dictionary Key Access
^^^^^^^^^^^^^^^^^^^^^

The dot ``.`` operator can be used to access the value associated with
a key in a dictionary or class instance. For example, the rule::

  first_name(dict, x) if
    dict = new Person{} and
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

By default, Polar runs all of the definitions for a given rule that are
applicable to the given set of arguments (i.e., whose specializers are
matched). The ``cut`` operator overrides this behavior by *committing* to
the enclosing rule definition: the query engine will not run any others.
Rule definitions that have already run are not "un-run", though, or avoided
by using cut; it just ensures that no *others* will run.

Because Polar runs rules in most-to-least-specific order, these "other"
rule definitions are always *less specific* than the current one; i.e.,
they may have specializers that are superclasses (and therefore less specific)
of those in the current rule. This allows ``cut`` to override a rule that
is specialized on a less specific class. You can think of ``cut`` as a sort
of dual to ``super()`` in other object-oriented languages (e.g., Python):
in Polar, the behavior of "methods" (rules) is to implicitly call the
next method, but ``cut`` overrides that behavior; it says *not* to call
any more methods (rules).

``cut`` can appear anywhere in a rule body, but terms before it must
succeed for it to be reached, so it frequently appears at the end of
the body: **if** so-and-so is true, then **cut** out all other alternatives.

``cut`` should be used sparingly.

.. _operator-new:

New
^^^

The ``new`` operator is used to construct a new instance of an application class.
See :doc:`/getting-started/policies/application-types`. The single argument to the
new operator must be an instance literal::

    new Person{first_name: "yogi", last_name: "bear"}

.. _operator-in:

In (List Membership)
^^^^^^^^^^^^^^^^^^^^

The ``in`` operator can be used to iterate over a list. If the second operand is a list, the first operand will
be unified with each element of the list. If the
second operand is not a list (or variable bound to a list),
the operation will fail.

For example::

    x in [1, 2, 3] and x = 1

Will bind ``x`` to ``1``, ``2``, ``3``, in turn, and check that ``x = 1``
for each. This expression will only succeed for the first item (``1``).

The ``in`` operator generates *alternatives* for each element of the list.

.. _operator-forall:

For All
^^^^^^^

The ``forall`` operator is often useful in conjunction with the ``in`` operator.
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
cannot be accessed after the ``forall`` operator.

.. _operator-rest:

``*rest`` Operator
^^^^^^^^^^^^^^^^^^

The rest operator (``*``) can be used to destructure a list. For example::

    x = [1, 2, 3] and
    [first, *tail] = x

After executing the above, the variable ``first`` will have the value ``1``, and
``tail`` the value ``[2, 3]``.

The rest operator is only valid within a list literal and in front of a
variable. It **must** be the last element of the list literal (``[*rest,
tail]``) is invalid. Any number of elements can come before the rest operator.

The rest operator is only useful when combined with a unification operation that
assigns a value to it.

Patterns and Matching
----------------------

Polar has powerful pattern matching facilities that are useful to control which
rules execute & in what order.

.. _specialization:

Specialization
^^^^^^^^^^^^^^

Rule heads (the part of the rule before the ``if`` keyword) can contain
specializers.  For example, the rule::

    has_first_name(person: Person, name) if person.name = name;

Would only execute if the ``person`` argument is of the type ``Person``.

Multiple rules of the same structure can be written with different
specializers::

    has_first_name(user: User, name) if user.name = name;

Now, the ``first_name`` rule can be used with instances of the ``User`` or
``Person`` type.

For more on this feature, see
:doc:`/getting-started/policies/application-types`.

.. _pattern:

Patterns
^^^^^^^^

The expression after the ``:`` is called a pattern.  The following are valid
patterns:

- any primitive type
- a dictionary literal
- an instance literal (without the new operator)
- a type name (used above)

When a rule is evaluated, the value of the argument is matched against the
pattern.  For primitive types, a value matches a pattern if it is equal.

For dictionary types, a value matches a pattern if the pattern is a subset of
the dictionary.  For example::

    {x: 1, y: 2} matches {x: 1}
    {x: 1, y: 3} matches {y: 3}
    {x: 1, y: 3} matches {x:1, y: 3}

    # Does not match because y value are not equal
    not {x: 1, y: 3} matches {x:1, y: 4}

    # a type name matches if the value has the same type
    new Person{} matches Person

    # The fields are checked in the same manner as dictionaries, and the type is
    # checked like above.
    new Person{x: 1, y: 2} matches Person{x: 1}

For type matching, subclasses are also considered.  So, a class that is a
subclass of ``Person`` would match ``Person{x: 1}``.

.. _operator-matches:

Matches Operator
^^^^^^^^^^^^^^^^

The above example used the ``matches`` operator to describe the behavior of
pattern matching.  This operator can be used anywhere within a rule body to
perform a match.  The same operation is used by the engine to test whether a
rule argument matches the specializer.

.. _inline-queries:

Inline Queries (``?=``)
-----------------------

Queries can also be added to Polar files and will run when the file is loaded.
Inline queries can be useful for testing a policy and confirming it behaves as
expected.

To add an inline query to a Polar file, use the ``?=`` operator::

    # policy.polar
    ?= allow("foo", "read", "bar")

An inline query is only valid at the beginning of a line.

Inline queries are particularly useful for testing policies.
