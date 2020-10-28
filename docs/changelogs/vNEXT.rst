====
NEXT
====

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
============

Feature 1
---------

- summary
- of
- user facing changes

Link to relevant documentation section

Other bugs & improvements
=========================

- We now check fields in the case of a ``matches`` against a built-in type. E.g.:

  .. code-block:: polar

    2 matches Integer { numerator: 2, denominator: 1 }
