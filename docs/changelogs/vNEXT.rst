====
NEXT
====

**Release date:** XXXX-XX-XX

Breaking changes
================

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.

``Oso.clear()`` replaced with ``Oso.clear_rules()``/``clearRules()``
--------------------------------------------------------------------

The ``Oso.clear()`` method in oso's language libraries has been removed.
To clear rules from the Polar knowledge base, use the new ``clear_rules()``
(or ``clearRules()``) method, which clears rules but leaves registered classes
and constants in place.

To migrate, replace calls to ``Oso.clear()`` with either ``Oso.clear_rules()`` or
``Oso.clearRules()``, depending on the library you are using.
It is no longer necessary to re-register classes/constants after clearing.

Method signature for ``Oso.register_constant()`` updated
--------------------------------------------------------

The parameters were swapped to mirror the signature of
``Oso.register_class()``.

Select Ruby methods now return ``self`` to enable method chaining
-----------------------------------------------------------------

- ``Oso#clear_rules``
- ``Oso#load_file``
- ``Oso#load_str``
- ``Oso#register_class``
- ``Oso#register_constant``

Custom constructors no longer supported in the Java, Python, or Ruby libraries
------------------------------------------------------------------------------

For the Java, Python, and Ruby libraries, custom constructors are a relic. They
were useful for translating keyword args into positional args before oso
supported supplying positional args when constructing an instance via Polar's
:ref:`new operator <operator-new>`. They were also useful for specifying a
``find_or_create``-style class method as a constructor, but that's been
superseded by the introduction of calling methods directly on registered
constants, including classes.

To migrate, replace usage of a custom constructor with an equivalent class
method.

Note that custom constructors are still supported for the Rust library since
specifying a static ``new`` method for a type is nothing more than a
convention.

New features
============

Feature 1
---------

- summary
- of
- user facing changes

Link to relevant documentation section


Other bugs & improvements
=========================

- Language libraries that haven't yet implemented operations on application
  instances (Java, Node.js, Ruby, Rust) now throw a uniform error type.

Improvements to the debugger
----------------------------

- Changes to the way stepping is implemented:
    - ``step`` steps by query instead of goal.
    - ``over`` and ``out`` are now implemented using a stack that tracks query
      parents.
- New ``goal`` command to step by goal (the way ``step`` used to work).
- New ``stack`` command to show all parent queries of the current one.
- ``query n`` command can take an integer argument ``n`` to inspect the query
  at the nth level of the stack.
