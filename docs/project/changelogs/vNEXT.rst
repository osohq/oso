=====
NEXT
=====

**Release date:** XXXX-XX-XX

Breaking changes
================

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.

isa operator replaced with matches
----------------------------------

To improve readability of Polar policies, the ``isa`` operator has
been replaced with ``matches`` (:ref:`operator-matches`). Any policies using the
``isa`` operator will need to be migrated.


New features
==============

Feature 1
=========

- summary
- of
- user facing changes

Link to relevant documentation section


Other bugs & improvements
=========================

- Boolean values can now be queried directly.  The statement ``x = true and x``
  now tests for the truthiness of ``x`` as the second argument of the
  conjunction. Previously this would be invalid.
