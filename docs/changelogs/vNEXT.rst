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


Removed "extras"
----------------

The oso library previously had some additional default supported classes:
``Http`` and ``Pathmapper``. These have been deprecated and removed.

To write policies over HTTP requests, either register the suitable
application class directly, or use a framework integration (e.g.
``flask-oso`` or ``django-oso``) which will do this for you
automatically.

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

- bulleted list
- improvements
- of smaller
- potentially with doc links
