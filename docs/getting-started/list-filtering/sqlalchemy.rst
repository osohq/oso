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

.. todo:: Formatting is unfortunate but continuing...

Let's test out the policy in a REPL:

.. mdinclude:: ../../examples/list-filtering/sqlalchemy/example.md

How oso authorizes SQLAlchemy Data
==================================

As you can see from the above example, the SQLAlchemy oso integration allows
regular SQLAlchemy queries to be executed with authorization applied.

To accomplish this, oso has a custom query class that applies authorization
filters based on the loaded policy before the query is executed.

Before compiling a SQLAlchemy query, the entities in the query are authorized
with oso.  oso returns authorization decisions for each entity that indicate
what constraints must be met for the entity to be authorized.  These constraints
are then translated into filters on the SQLAlchemy query object.

For example, our above policy has the following code:

.. code-block:: polar

    allow(user: User, "read", post: Post) if
        post.access_level = "private" and post.created_by = user;

The oso library converts the constraints on Post expressed in this policy into a
SQLAlchemy query like::

    session.query(Post)
        .filter(Post.access_level == "private" & Post.created_by == user)

This translation makes the policy an effective abstraction for expressing
policies that must be enforced over objects stored in a database.

Limitations
===========

This feature is still under active development. Most unsupported policies will
cause a runtime error rather than incorrect authorization behavior. More
policies will be supported as we continue working on this feature.  The
SQLAlchemy adaptor is ready for evaluation and testing. However, we recommending
getting in touch with us before using it in production.  Join our Slack_.

There are some operators and features of Polar that do not currently work with
the SQLAlchemy Library when used **anywhere in the policy**:

- the ``cut`` operator
- rules that rely on ordered execution based on class inheritance
- negated queries using the ``not`` operator that contain a ``matches`` operator
  within the negation

Some operations cannot be performed on **resources** in ``allow`` rules used with
the SQLAlchemy adaptor.  These operations can still be used on the actor or
action:

- application method calls
- the ``matches`` operator with fields (``x matches Foo{a: 1}``).
- rule specializers with fields (``allow(_, _, _: Foo{a: 1}) if ...;``)
- arithmetic operators

.. _Slack: http://join-slack.osohq.com/
