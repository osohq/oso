=====
NEXT
=====

**Release date:** XXXX-XX-XX

Breaking changes
================

.. TODO remove warning and replace with "None" if no breaking
   changes.

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.

Breaking change 1
-----------------

- summary of breaking change

Link to migration guide


New features
==============

Windows Support
===============
The three oso libraries (Python, Ruby and Java) all now work on Windows.

musl builds for Python
======================

musl-based Python wheels (for Alpine Linux and other musl-based distros) are
built and downloadable from `the releases page on GitHub
<https://github.com/osohq/oso/releases/latest>`_.

Assignment Operator
===================
- The operator ``:=`` may now be used to assign values to unbound variables.
  Unlike the unify operator (``=``), the assignment operator will NOT evaluate
  equality.
- Attempting to assign to a non-variable will result in a parse error.
- Attempting to assign to a bound variable will result in a runtime error.

Built-in Types
==============

You may now write rules that specialize on any of the built-in types
``Boolean``, ``Integer``, ``Float``, ``List``, ``Dictionary``, and ``String``.
These types are mapped to host-language classes such as ``java.lang.Boolean``
in Java or ``bool`` in Python.

Positional Arguments to Constructors
====================================

The ``new`` operator previously required an instance literal whose fields
are passed to the class's constructor as keyword arguments:

.. code-block:: polar

    new Person{first: "First", last: "Last"}

This syntax is still supported in application languages that support keyword
arguments (e.g., Python and Ruby), but some languages (e.g., Java) do not
support keywords. So a new syntax was added to pass initialization arguments
positionally:

.. code-block:: polar

    new Person("First", "Last")

Positional constructor arguments may be used in any application language.

Java Class Registration
=======================
The Java ``registerClass`` method now requires only a class:

.. code-block:: Java

    registerClass(Person.class)

If you want to always use a specific constructor from within
a policy, you may now specify a ``Constructor`` to use:

.. code-block:: Java

    registerClass(Person.class, Person.class.getConstructor(String.class, String.class))

This takes the place of the function previously required to map keyword
arguments to positional ones.

If you omit the constructor (recommended), the default behavior at
instantiation time is to search the list returned by ``Class.getConstructors``
for a constructor that is applicable to the supplied (positional) constructor
arguments; see :doc:`/using/libraries/java/index` for details.

Flask Integration (``flask_oso``)
==================================

The new flask_oso_ package makes it easy to use oso with Flask, the popular
Python web framework. It includes a flask-specific authorization method with
sensible defaults, middleware that ensure all requests are properly authorized,
and route decorators to more succinctly use oso.

.. code-block:: python

    from flask_oso import authorize

    @authorize(resource="get_user")
    @app.route("/user")
    def get_user():
        return "current user"

Other bugs & improvements
=========================

- fixed float parsing
- improved integer/float comparisons
- Fix checking membership in an empty list. ``x in []`` is now always false
- fixed bug causing memory issues when running across multiple threads
