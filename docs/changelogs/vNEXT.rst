====
NEXT
====

**Release date:** XXXX-XX-XX

Breaking changes
================

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.

allow() method renamed in oso libraries
---------------------------------------

To eliminate ambiguity, the ``allow()`` method of the oso library has been renamed to:

- ``is_allowed()`` in Python,
- ``#allowed?`` in Ruby, and
- ``isAllowed()`` in Java

isa operator replaced with matches
----------------------------------

To improve readability of Polar policies, the ``isa`` operator has
been replaced with ``matches`` (:ref:`operator-matches`). Any policies using the
``isa`` operator will need to be migrated.

cut() predicate replaced with cut keyword
-----------------------------------------

The Polar "cut" operator was previously parsed like a predicate, but
it isn't one. The new syntax ``cut`` (without the empty parentheses)
emphasizes its role as a keyword.

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
