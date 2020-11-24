==================
SQLAlchemy Adapter
==================

The ``sqlalchemy_oso`` library can enforce policies over SQLAlchemy models.
This allows policies to enforce access to collections of objects without
needing to authorize each object individually.

Usage
=====

.. todo:: (dhatch) More precise terminology (declarative models, etc.) in this
   section.

The ``sqlalchemy_oso`` library works over your existing SQLAlchemy ORM models
without modification.

To get started, we need to:

1. Make oso aware of our SQLAlchemy model types so that we can write policies
   over them.
2. Create a SQLAlchemy Session_ that uses oso to authorize access to data.

**Register models with oso**

We write a policy over our SQLAlchemy models.
:py:func:`sqlalchemy_oso.register_models` registers all models
that descend from a declarative base class as types that are available
in the policy.

Alternatively, the :py:meth:`oso.Oso.register_class` method can be called on
each SQLAlchemy model that you will write rules for.

**Create a SQLAlchemy Session that uses oso**

oso performs authorization by integrating with SQLAlchemy sessions.
Use the :py:func:`sqlalchemy_oso.authorized_sessionmaker` session
factory instead of the default SQLAlchemy ``sessionmaker``. Every query made
using sessions from this factory will have authorization applied.

Before executing a query, oso consults the policy and obtains a list of
conditions that must be met for an object to be authorized.  These conditions
are translated into SQLAlchemy expressions and added to the query before
retrieving objects from the database.

.. _Session: https://docs.sqlalchemy.org/en/13/orm/session_api.html#sqlalchemy.orm.session.Session

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

.. note::

    The SQLAlchemy integration is deny by default.  The final rule for ``User``
    is needed to allow access to user objects for any user.

    If a query is made for a model that does not have an explict rule in the
    policy, no results will be returned.

These rules are written over single model objects.

.. todo:: Formatting is unfortunate but continuing...

Let's test out the policy in a REPL:

.. mdinclude:: ../../examples/list-filtering/sqlalchemy/example.md

How oso authorizes SQLAlchemy Data
==================================

As you can see from the above example, the SQLAlchemy oso integration allows
regular SQLAlchemy queries to be executed with authorization applied.

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
authorization logic over collections.

Limitations
===========

This feature is still under active development. Not all policies that work in a
non-partial setting will currently work with partials. More policies will be
supported as we continue working on this feature.  The SQLAlchemy adapter is
ready for evaluation and testing. However, we recommending getting in touch with
us before using it in production.  Join our Slack_.

There are some operators and features of Polar that do not currently work with
the SQLAlchemy Library when used **anywhere in the policy**:

- the ``cut`` operator
- rules that rely on ordered execution based on class inheritance
- negated queries using the ``not`` operator that contain a ``matches`` operator
  within the negation or call a rule that contains a specializer. For example:

  .. code-block:: polar

        # Not supported.
        allow(actor, action, resource) if
            not resource matches User;

        # Also not supported.
        is_user(user: User);
        allow(actor, action, resource) if
            not is_user(resource);

Some operations cannot be performed on **resources** in ``allow`` rules used
with the SQLAlchemy adapter.  These operations can still be used on the actor or
action:

- application method calls
- arithmetic operators
- comparison operators

.. _Slack: http://join-slack.osohq.com/
