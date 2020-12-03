==================
SQLAlchemy Adapter
==================

The ``sqlalchemy-oso`` library can enforce policies over SQLAlchemy models.
This allows policies to control access to collections of objects without
needing to authorize each object individually.

Usage
=====

.. todo:: More precise terminology (declarative models, etc.) in this
   section.

``sqlalchemy-oso`` works over your existing SQLAlchemy ORM models
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
conditions that must be met for a model to be authorized. These conditions
are translated into SQLAlchemy expressions and applied to the query before
retrieving objects from the database.

.. _Session: https://docs.sqlalchemy.org/en/13/orm/session_api.html#sqlalchemy.orm.session.Session

Example
=======

Let's look at an example usage of this library. Our example is a social media
app that allows users to create posts. There is a ``Post`` model and a ``User``
model:

.. literalinclude:: /examples/list-filtering/sqlalchemy/sqlalchemy_example/models.py
    :caption: :fab:`python` models.py
    :language: python


Now, we'll write a policy over these models. Our policy contains the following
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

    The SQLAlchemy integration is deny by default. The final rule for ``User``
    is needed to allow access to user objects for any user.

    If a query is made for a model that does not have an explicit rule in the
    policy, no results will be returned.

These rules are written over single model objects.

Trying it out
-------------

Let's test out the policy in a REPL.

.. testsetup::

    import os

    os.chdir("examples/list-filtering/sqlalchemy")

First, import ``sqlalchemy``, ``oso`` and ``sqlalchemy_oso``.

.. doctest::

    >>> from sqlalchemy import create_engine
    >>> from sqlalchemy.orm import Session
    >>> from oso import Oso
    >>> from sqlalchemy_oso import authorized_sessionmaker, register_models
    >>> from sqlalchemy_example.models import Model, User, Post

Then, setup ``oso`` and register our models.

.. doctest::

    >>> oso = Oso()
    >>> register_models(oso, Model)
    >>> oso.load_file("sqlalchemy_example/policy.polar")

Next, setup some test data...

.. doctest::

    >>> user = User(username='user')
    >>> manager = User(username='manager', manages=[user])

    >>> public_user_post = Post(contents='public_user_post',
    ...                         access_level='public',
    ...                         created_by=user)
    >>> private_user_post = Post(contents='private_user_post',
    ...                          access_level='private',
    ...                          created_by=user)
    >>> private_manager_post = Post(contents='private_manager_post',
    ...                             access_level='private',
    ...                             created_by=manager)
    >>> public_manager_post = Post(contents='public_manager_post',
    ...                            access_level='public',
    ...                            created_by=manager)

... and load that data into SQLAlchemy.

.. doctest::

    >>> engine = create_engine('sqlite:///:memory:')
    >>> Model.metadata.create_all(engine)
    >>> fixture_session = Session(bind=engine)
    >>> fixture_session.add_all([
    ...     user, manager, public_user_post, private_user_post, private_manager_post,
    ...     public_manager_post])
    >>> fixture_session.commit()

**Authorizing user's posts**

Now that we've setup some test data, let's use **oso** to authorize
``Post``\ s that ``User(username="user")`` can see.

We'll start by making an :py:func:`~sqlalchemy_oso.authorized_sessionmaker`.

.. doctest::

   >>> AuthorizedSession = authorized_sessionmaker(bind=engine,
   ...                                             get_oso=lambda: oso,
   ...                                             get_user=lambda: user,
   ...                                             get_action=lambda: "read")
   >>> session = AuthorizedSession()

Then, issue a query for all posts.

.. doctest::

    >>> posts = session.query(Post).all()
    >>> [p.contents for p in posts]
    ['public_user_post', 'private_user_post', 'public_manager_post']


Since we used :py:func:`~sqlalchemy_oso.authorized_sessionmaker`, the query only
returned authorized posts based on the policy.

``user`` can see their own public and private posts, and other public
posts.

**Authorizing manager's posts**

Now we’ll authorize access to ``manager``\ 's ``Post``\ s. We create a new
authorized session with user set to manager.

.. doctest::

    >>> AuthorizedSession = authorized_sessionmaker(bind=engine,
    ...                                             get_oso=lambda: oso,
    ...                                             get_user=lambda: manager,
    ...                                             get_action=lambda: "read")
    >>> manager_session = AuthorizedSession()

.. note::

    In a real application, ``get_user`` would be a function returning the
    current user, based on the current request context. For example in Flask
    this might be ``lambda: flask.g.current_user`` or some other proxy object.


And issue the same query as before...

.. doctest::

    >>> posts = manager_session.query(Post).all()
    >>> [p.contents for p in posts]
    ['public_user_post', 'private_user_post', 'private_manager_post', 'public_manager_post']

This time, the query returned four posts! Since the ``manager`` user manages
``user``, the private post of user is also authorized (based on our third rule
above).

.. doctest::

   >>> manager.manages[0].username
   'user'


This full example is available on GitHub_.

.. _GitHub: https://github.com/osohq/oso/tree/main/docs/examples/list-filtering/sqlalchemy

How oso authorizes SQLAlchemy Data
==================================

As you can see from the above example, the SQLAlchemy oso integration allows
regular SQLAlchemy queries to be executed with authorization applied.

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

.. todo:: Wording here still needs a bit of work once the top level page is
    in, and should link back to that page. It might not be clear what
    "non-partial" setting means just from reading this page.

This feature is still under active development. Not all policies that work in a
non-partial setting will currently work with partials. More policies will be
supported as we continue working on this feature. The SQLAlchemy adapter is
ready for evaluation and testing. However, we recommend getting in touch with
us on Slack_ before using it in production.

There are some operators and features that do not currently work with the
SQLAlchemy adapter when used **anywhere in the policy**:

- the ``cut`` operator
- rules that rely on ordered execution based on class inheritance
- negated queries using the ``not`` operator that contain a ``matches``
  operation within the negation or call a rule that contains a specializer. For
  example:

  .. code-block:: polar

        # Not supported.
        allow(actor, action, resource) if
            not resource matches User;

        # Also not supported.
        is_user(user: User);
        allow(actor, action, resource) if
            not is_user(resource);

Some operations cannot be performed on **resources** in ``allow`` rules used
with the SQLAlchemy adapter. These operations can still be used on the actor or
action:

- application method calls
- arithmetic operators

.. _Slack: http://join-slack.osohq.com/
