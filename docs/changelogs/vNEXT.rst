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

List filtering in ``django-oso`` (preview)
-------------------------------------------

oso can now respond to some queries with a set of constraints instead of a
yes or no decision.  In the ``django-oso`` library, the
:py:meth:`django_oso.auth.authorize_model` function and
:py:class:`django_oso.models.AuthorizedModel` class have been added to use this
functionality to authorize a **collection** of objects.  Instead of fetching all
objects and evaluating a query, the relevant authorization constraints will be
pushed down to the ORM and applied to a Django ``QuerySet``.

This feature makes implementing list endpoints with authorization more
performant, since authorization does not need to be applied after fetching data.

**This feature is currently in preview.**


.. todo:: Link to blog post


Other bugs & improvements
=========================

- bulleted list
- improvements
- of smaller
- potentially with doc links
