.. title:: Changelog for Release 2021-01-06
.. meta::
  :description: Changelog for Release 2021-01-06 (oso 0.10.0) containing new features, bug fixes, and more.

##################
Release 2021-01-06
##################

=============
``oso`` 0.10.0
=============

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

``get_allowed_actions`` introduced for Python
---------------------------------------------

Use :py:meth:`Oso.get_allowed_actions` to get a list of actions that a user
is allowed to take on a resource. These actions can be used for making
additional authorization decisions, especially in the frontend (e.g., hiding
or showing a button based on the current user's allowed actions).

See our guide :doc:`here<TODO>`!

``PolarClass`` implemented for versions 0.7 & 0.8 of the ``uuid`` crate
-----------------------------------------------------------------------

``PolarClass`` is now implemented for versions 0.7 & 0.8 of the ``uuid`` crate
behind the optional ``uuid-07`` feature flag.

Ruby library now supports Ruby 3.0
----------------------------------

There are no breaking changes. Happy Rubying!

Other bugs & improvements
=========================

- bulleted list
- improvements
- of smaller
- potentially with doc links
