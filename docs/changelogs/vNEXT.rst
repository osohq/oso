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


Other bugs & improvements
=========================

- fixed float parsing
- improved integer/float comparisons
