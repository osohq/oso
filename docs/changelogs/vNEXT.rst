=====
0.8.3
=====

**Release date:** 2020-12-08

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

``PolarClass`` implemented for ``uuid`` crate
---------------------------------------------

``PolarClass`` is now implemented for `version 0.6 of the uuid crate
<https://docs.rs/uuid/0.6/uuid/>`_ behind the optional ``uuid-06`` feature
flag. Version 0.6 was chosen for compatibility with `Diesel
<https://crates.io/crates/diesel>`_.

Thanks to `John Halbert <https://github.com/johnhalbert>`_ for the
contribution!

Other bugs & improvements
=========================

- ``matches`` operations on fields of partials are now handled correctly in the
  SQLAlchemy adapter. Previously these operations would result in an error.
- The SQLAlchemy list filtering adapter now supports all comparisons. Previously
  comparisons other than ``==`` or ``=`` would cause an error.
- Fixed bug where checking if a character is in a string would fail incorrectly.
