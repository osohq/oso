=====
0.8.3
=====

**Release date:** 2020-12-08

Breaking changes
================

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.

Simplified ``sqlalchemy-oso`` session creation
----------------------------------------------

``sqlalchemy-oso`` now associates the current oso instance, user to authorize,
and action to authorize with
:py:class:`sqlalchemy_oso.session.AuthorizedSession`.  This class manages
authorization instead of the removed
``sqlalchemy_oso.hooks.make_authorized_query_cls``.

- The ``sqlalchemy.hooks`` module has been renamed to ``sqlalchemy.session``.
  Update any imports to ``sqlalchemy.session``.
- The ``sqlalchemy_hooks.make_authorized_query_cls`` function has been removed.
  Use the session API instead
  (:py:func:`sqlalchemy_oso.authorized_sessionmaker`).
- The ``sqlalchemy_oso.authorized_sessionmaker`` function no longer accepts
  extra positional arguments. Use keyword arguments to pass options to the
  session.


Removed "extras"
----------------

The oso library previously had some additional default supported classes:
``Http`` and ``Pathmapper``. These have been deprecated and removed.

Instead, you should register suitable classes directly. For example,
registering the ``werkzeug.wrappers.Request`` object for Flask applications.

New features
============

``PolarClass`` implemented for ``uuid`` crate
---------------------------------------------

``PolarClass`` is now implemented for `version 0.6 of the uuid crate
<https://docs.rs/uuid/0.6/uuid/>`_ behind the optional ``uuid-06`` feature
flag. Version 0.6 was chosen for compatibility with `Diesel
<https://crates.io/crates/diesel>`_.

Thanks to `John Halbert <https://github.com/johnhalbert>`_ for the
contribution!

Improved ``sqlalchemy-oso`` support for usage with ``flask_sqlalchemy``
-----------------------------------------------------------------------

The ``sqlalchemy-oso`` library now has a built-in wrapper class that makes it
easier to use with the popular ``flask_sqlalchemy`` library.  See
:py:class:`sqlalchemy_oso.flask.AuthorizedSQLAlchemy` for more information.

``scoped_session`` support for ``sqlalchemy-oso``
-------------------------------------------------

The new :py:func:`sqlalchemy_oso.session.scoped_session` session proxy can be
used instead of SQLAlchemy's built-in scoped_session_.  This creates a session
scoped to the current oso instance, user and action.

.. _scoped_session: https://docs.sqlalchemy.org/en/13/orm/contextual.html#sqlalchemy.orm.scoping.scoped_session

Other bugs & improvements
=========================

- ``matches`` operations on fields of partials are now handled correctly in the
  SQLAlchemy adapter. Previously these operations would result in an error.
- The SQLAlchemy list filtering adapter now supports all comparisons. Previously
  comparisons other than ``==`` or ``=`` would cause an error.
- The Django list filtering adapter now fully supports use of the ``not``
  operator in policies.
- Fixed bug where checking if a character is in a string would fail incorrectly.
- Relaxed the requirements for Python ``oso`` and ``django-oso`` libraries.
  These now require ``cffi~=1.14`` and ``django>=2.2`` respectively.
