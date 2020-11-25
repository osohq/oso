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
app that allows users to view posts. There is a ``User`` model and a ``Post``
model:

.. literalinclude:: /examples/list-filtering/django/example/app/models.py
    :caption: :fab:`python` models.py
    :language: python

We want to enforce the following authorization scheme for posts:

1. Anyone is allowed to ``GET`` any public post.
2. A user is allowed to ``GET`` their own private posts.
3. A user is allowed to ``GET`` private posts made by users they manage
   (defined through the ``user.manager`` relationship).

The corresponding policy looks as follows:

.. literalinclude:: /examples/list-filtering/django/example/app/policy/example.polar
    :caption: :fa:`oso` example.polar
    :language: polar

Trying it out
-------------

If you want to follow along, clone the oso repository from GitHub_ and ``cd``
into the ``docs/examples/list-filtering/django`` directory. Then, run ``make
setup`` to install dependencies (primarily Django and ``django-oso``) and seed
the database.

.. _GitHub: https://github.com/osohq/oso

The database will now contain a set of four posts made by two users:

.. code-block:: python

    manager = User(username="manager")
    user = User(username="user", manager=manager)

    Post(contents="public user post", access_level="public", creator=user)
    Post(contents="private user post", access_level="private", creator=user)
    Post(contents="public manager post", access_level="public", creator=manager)
    Post(contents="private manager post", access_level="private", creator=manager)

Once everything is set up, run ``python example/manage.py runserver`` to start
the Django app. We can now use cURL to interact with the application.

A guest user may view public posts:

.. code-block:: console

    $ curl localhost:8000/posts
    1 - @user - public - public user post
    3 - @manager - public - public manager post

A non-manager may view public posts and their own private posts:

.. code-block:: console

    $ curl --user user:user localhost:8000/posts
    1 - @user - public - public user post
    2 - @user - private - private user post
    3 - @manager - public - public manager post

A manager may view public posts, their own private posts, and private posts of
their direct reports:

.. code-block:: console

    $ curl --user manager:manager localhost:8000/posts
    1 - @user - public - public user post
    2 - @user - private - private user post
    3 - @manager - public - public manager post
    4 - @manager - private - private manager post

How oso authorizes Django data
==============================

As you can see from the above example, the ``django-oso`` integration applies
authorization to regular Django QuerySets.

Before evaluating a Django QuerySet, the authorized models in the QuerySet are...

Before compiling a SQLAlchemy query, the entities in the query are authorized
with oso. oso returns authorization decisions for each entity that indicate
what constraints must be met for the entity to be authorized. These constraints
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

This feature is still under active development. Not all valid policies are
currently supported, but more will be supported as we continue working on this
feature. The Django adapter is ready for evaluation and testing. However, we
recommend getting in touch with us on Slack_ before using it in production.

There are some operators and features that do not currently work with the
Django adapter when used **anywhere in the policy**:

- The ``cut`` operator.
- Rules that rely on ordered execution based on class inheritance.

Some operations cannot be performed on **authorized models** in rules used with
the Django adapter. These operations can still be used on regular Django models
or Python objects:

- Application method calls.
- Arithmetic operators.
- Negating (with ``not``) a ``matches`` operation whose left-hand side is an
  authorized model or a call to a rule that specializes on an authorized model.
  For example, if ``resource`` is an ``AuthorizedModel``:

  .. code-block:: polar

        # Not supported.
        allow(actor, action, resource) if
            not resource matches Post;

        # Also not supported.
        is_post(_: Post);
        allow(actor, action, resource) if
            not is_post(resource);

.. _Slack: http://join-slack.osohq.com/
