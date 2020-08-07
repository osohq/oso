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

Assignment Operator
===================
- The operator ``:=`` may now be used to assign values to unbound variables. Unlike the unify operator (``=``),
the assignment operator will NOT evaluate equality.
- Attempting to assign to a non-variable will result in a parse error.
- Attempting to assign to a bound variable will result in a runtime error.

Built-in Types
==============

You may now write rules that specialize on any of the built-in types
``Boolean``, ``Integer``, ``Float``, ``List``, ``Dictionary``, and ``String``.
These types are mapped to host-language classes such as ``java.lang.Boolean``
in Java or ``bool`` in Python.

Other bugs & improvements
=========================

- fixed float parsing
- improved integer/float comparisons
- Fix checking membership in an empty list. ``x in []`` is now always false
