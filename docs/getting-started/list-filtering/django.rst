==============
Django Adapter
==============

The ``django-oso`` library can enforce policies over Django models. This allows
policies to control access to collections of objects without needing to
authorize each object individually.

Usage
=====

The ``django-oso`` library works over your existing Django models without
modification. The policies you write will largely look the same with or without
the list filtering feature, and the oso engine will follow similar evaluation
paths.

The main difference is in the API through which you request an authorization
decision:

  - :py:func:`django_oso.auth.authorize` to authorize access to a particular
    instance (the non-list filtering case).
  - :py:func:`django_oso.auth.authorize_model` to authorize access over a model
    (the list filtering case).

In the list filtering case, oso consults the policy to build up a list of
conditions that must be met in order for a model to be authorized. These
conditions are translated into Django ORM filters and applied to the query
before retrieving objects from the database.
