==================
SQLAlchemy Adaptor
==================

The ``sqlalchemy_oso`` library can enforce policies over SQLAlchemy models.
This allows policies to enforce access to collections of objects without
needing to authorize each object individually.

Usage
=====

.. todo:: (dhatch) More precise terminology (declarative models, etc.) in this
   section.

The ``sqlalchemy_oso`` library works over your existing SQLAlchemy ORM models
without modification.  To get started, we write a policy over our model types.
:py:func:`sqlalchemy.auth.register_models` can be used to register all models
that descend from a declarative base class with oso as types that are available
in the policy.

Then, use the :py:func:`sqlalchemy.hooks.authorized_sessionmaker` session
factory instead of the default SQLAlchemy ``sessionmaker``. Every query made
using sessions from this factory will have authorization applied.

Before executing a query, oso consults the policy and obtains a list of
conditions that must be met for an object to be authorized.  These conditions
are translated into SQLAlchemy expressions and added to the query before
retrieving objects from the database.

Example
=======

Let's look at an example usage of this library.  Our example is a social media
app that allows users to create posts.  There is a ``Post`` model and a ``User``
model:

.. literalinclude:: /examples/list-filtering/sqlalchemy/sqlalchemy_example/models.py
    :caption: :fab:`python` models.py
    :language: python


Now, we'll write a policy over these models.  Our policy contains the following
rules:

1. A user can read any public post.
2. A user can read their own private posts.
3. A user can read private posts for users they manage (defined through the
   ``user.manages`` relationship).
4. A user can read all other users.

.. literalinclude:: /examples/list-filtering/sqlalchemy/sqlalchemy_example/policy.polar
    :caption: :fa:`oso` policy.polar
    :language: polar

Note that these rules are written over single model objects.

Let's test out the policy in a REPL::
