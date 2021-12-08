"""SQLAlchemy async session classes and factories for oso."""
from typing import Any, Callable, Optional, Type
import logging

from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy_oso.session import Permissions, AuthorizedSessionBase
from sqlalchemy import orm

from oso import Oso

logger = logging.getLogger(__name__)


def async_authorized_sessionmaker(
    get_oso: Callable[[], Oso],
    get_user: Callable[[], Any],
    get_checked_permissions: Callable[[], Permissions],
    class_: Type[AsyncSession] = None,
    **kwargs,
):
    """AsyncSession factory for sessions with Oso authorization applied.

    :param get_oso: Callable that returns the Oso instance to use for
                    authorization.
    :param get_user: Callable that returns the user for an authorization
                     request.
    :param get_checked_permissions: Callable that returns an optional map of
                                    permissions (resource-action pairs) to
                                    authorize for the session. If the callable
                                    returns ``None``, no authorization will
                                    be applied to the session. If a map of
                                    permissions is provided, querying for
                                    a SQLAlchemy model present in the map
                                    will authorize results according to the
                                    action specified as the value in the
                                    map. E.g., providing a map of ``{Post:
                                    "read", User: "view"}`` where ``Post`` and
                                    ``User`` are SQLAlchemy models will apply
                                    authorization to ``session.query(Post)``
                                    and ``session.query(User)`` such that
                                    only ``Post`` objects that the user can
                                    ``"read"`` and ``User`` objects that the
                                    user can ``"view"`` are fetched from the
                                    database.
    :param class_: Base class to use for sessions.

    All other keyword arguments are passed through to
    :py:func:`sqlalchemy.orm.session.sessionmaker` unchanged.

    **Invariant**: the values returned by the `get_oso()`, `get_user()`, and
    `get_checked_permissions()` callables provided to this function *must
    remain fixed for a given session*. This prevents authorization responses
    from changing, ensuring that the session's identity map never contains
    unauthorized objects.
    """
    if class_ is None:
        class_ = AsyncSession

    # Oso, user, and checked permissions must remain unchanged for the entire
    # session. This is to prevent unauthorized objects from ending up in the
    # session's identity map.
    class Sess(AuthorizedSessionBase, orm.Session):  # type: ignore
        def __init__(self, **options):
            options.setdefault("oso", get_oso())
            options.setdefault("user", get_user())
            options.setdefault("checked_permissions", get_checked_permissions())
            super().__init__(**options)

    # We call sessionmaker here because sessionmaker adds a configure
    # method to the returned session and we want to replicate that
    # functionality.
    return orm.sessionmaker(class_=class_, sync_session_class=Sess, **kwargs)


def async_scoped_session(
    get_oso: Callable[[], Oso],
    get_user: Callable[[], Any],
    get_checked_permissions: Callable[[], Permissions],
    scopefunc: Optional[Callable[..., Any]] = None,
    **kwargs,
):
    """Return a async scoped session maker that uses the Oso instance, user, and
    checked permissions (resource-action pairs) as part of the scope function.

    Use in place of sqlalchemy's scoped_session_.

    Uses :py:func:`authorized_sessionmaker` as the factory.

    :param get_oso: Callable that returns the Oso instance to use for
                    authorization.
    :param get_user: Callable that returns the user for an authorization
                     request.
    :param get_checked_permissions: Callable that returns an optional map of
                                    permissions (resource-action pairs) to
                                    authorize for the session. If the callable
                                    returns ``None``, no authorization will
                                    be applied to the session. If a map of
                                    permissions is provided, querying for
                                    a SQLAlchemy model present in the map
                                    will authorize results according to the
                                    action specified as the value in the
                                    map. E.g., providing a map of ``{Post:
                                    "read", User: "view"}`` where ``Post`` and
                                    ``User`` are SQLAlchemy models will apply
                                    authorization to ``session.query(Post)``
                                    and ``session.query(User)`` such that
                                    only ``Post`` objects that the user can
                                    ``"read"`` and ``User`` objects that the
                                    user can ``"view"`` are fetched from the
                                    database.
    :param scopefunc: Additional scope function to use for scoping sessions.
                      Output will be combined with the Oso, permissions
                      (resource-action pairs), and user objects.
    :param kwargs: Additional keyword arguments to pass to
                   :py:func:`authorized_sessionmaker`.

    NOTE: _baked_queries are disabled on SQLAlchemy 1.3 since the caching
          mechanism can bypass authorization by using queries from the cache
          that were previously baked without authorization applied. Note that
          _baked_queries are deprecated as of SQLAlchemy 1.4.

    .. _scoped_session: https://docs.sqlalchemy.org/en/13/orm/contextual.html

    .. _baked_queries: https://docs.sqlalchemy.org/en/14/orm/extensions/baked.html
    """
    scopefunc = scopefunc or (lambda: None)

    def _scopefunc():
        checked_permissions = frozenset(get_checked_permissions().items())
        return (get_oso(), checked_permissions, get_user(), scopefunc())

    factory = async_authorized_sessionmaker(
        get_oso, get_user, get_checked_permissions, **kwargs
    )

    return orm.scoped_session(factory, scopefunc=_scopefunc)
