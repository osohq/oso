.. title:: Changelog for Release DATE
.. meta::
  :description: Changelog for Release DATE (RELEASED_VERSIONS) containing new features, bug fixes, and more.

############
Release DATE
############

==================================
``RELEASED_PACKAGE_1`` NEW_VERSION
==================================

Breaking changes
================

.. TODO remove warning and replace with "None" if no breaking
   changes.

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.


Breaking change 1
-----------------

1 Behavior of `a(x) if x` has changed
   * Now equivalant to `a(x) if x == true`
   * Now works if x is unbound

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

- The Polar `Variable` type is now exposed in the Node.js library, allowing users to pass unbound variables to `queryRule()` and `isAllowed()`.
- Go lib no longer tries to print the zero values it uses for bookkeeping. This would crash when running on macOS under delve.
