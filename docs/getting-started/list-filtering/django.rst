==============
Django Adapter
==============

The ``django-oso`` library can enforce policies over Django models. This allows
policies to control access to collections of objects without needing to
authorize each object individually.

Usage
=====

The easiest way to prepare your existing Django models for use in a list
filtering policy is to have them inherit from
:py:func:`django_oso.models.AuthorizedModel`, a thin wrapper around
``django.models.Model`` that calls :py:func:`django_oso.auth.authorize_model`
under the hood to return Django QuerySets with authorization filters applied.

The policies you write will largely look the same with or without the list
filtering feature, and the oso engine will follow similar evaluation paths.

In the list filtering case, oso consults the policy to build up a list of
conditions that must be met in order for a model to be authorized. These
conditions are translated into Django ORM filters and applied to the query
before retrieving objects from the database.

Example
=======

Let’s look at an example usage of this library. Our example is a social media
app that allows users to create posts. There is a ``Post`` model and a ``User``
model:

.. literalinclude:: /examples/list-filtering/django/example/models.py
    :caption: :fab:`python` models.py
    :language: python

Now, we'll write a policy over these models.  Our policy contains the following
rules:

1. A user can read any public post.
2. A user can read their own private posts.
3. A user can read private posts made by users they manage (defined through the
   ``user.manager`` relationship).

.. literalinclude:: /examples/list-filtering/django/example/policy/example.polar
    :caption: :fa:`oso` example.polar
    :language: polar

Trying it out
-------------

This full example is available on GitHub_.

.. _GitHub: https://github.com/osohq/oso/tree/main/docs/examples/list-filtering/django

How oso authorizes Django data
==============================

Limitations
===========
