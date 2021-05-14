"""SQLAlchemy session classes and factories for oso."""
from typing import Any, Callable, Dict, Optional, Type

from sqlalchemy import event, inspect
from sqlalchemy.orm.query import Query
from sqlalchemy.orm import sessionmaker, Session
from sqlalchemy.orm.util import AliasedClass
from sqlalchemy import orm
from sqlalchemy.sql import expression

from oso import Oso

from sqlalchemy_oso.auth import authorize_model


class _OsoSession:
    set = False

    @classmethod
    def get(cls):
        session = cls._get()
        new_session = Session(bind=session.bind)
        return new_session

    @classmethod
    def set_get_session(cls, get_session):
        cls.set = True
        _OsoSession._get = get_session


def set_get_session(oso: Oso, get_session_func):
    """Set the function that oso uses to expose a SQLAlchemy session to the policy

    :param oso: The Oso instance used to evaluate the policy.
    :type oso: Oso

    :param get_session_func: A function that returns a SQLAlchemy session
    :type get_session_func: lambda

    The session can be accessed from polar via the OsoSession constant. E.g.,

    .. code-block:: polar

        OsoSession.get().query(...)
    """
    _OsoSession.set_get_session(get_session_func)
    oso.register_constant(_OsoSession, "OsoSession")


@event.listens_for(Query, "before_compile", retval=True)
def _before_compile(query):
    """Enable before compile hook."""
    return _authorize_query(query)


Permissions = Optional[Dict[Type[Any], Any]]


def _authorize_query(query: Query) -> Optional[Query]:
    """Authorize an existing query with an Oso instance, user, and a
    permissions map indicating which actions to check for which SQLAlchemy
    models."""
    session = query.session

    # Early return if this isn't an authorized session.
    if not isinstance(session, AuthorizedSessionBase):
        return None

    oso: Oso = session.oso_context["oso"]
    user = session.oso_context["user"]
    checked_permissions: Permissions = session.oso_context["checked_permissions"]

    # Early return if no authorization is to be applied.
    if checked_permissions is None:
        return None

    # TODO (dhatch): This is necessary to allow ``authorize_query`` to work
    # on queries that have already been made.  If a query has a LIMIT or OFFSET
    # applied, SQLAlchemy will by default throw an error if filters are applied.
    # This prevents these errors from occuring, but could result in some
    # incorrect queries. We should remove this if possible.
    query = query.enable_assertions(False)  # type: ignore

    entities = {column["entity"] for column in query.column_descriptions}
    for entity in entities:
        # Only apply authorization to columns that represent a mapper entity.
        if entity is None:
            continue

        # If entity is an alias, get the action for the underlying class.
        if isinstance(entity, AliasedClass):
            action = checked_permissions.get(inspect(entity).class_)  # type: ignore
        else:
            action = checked_permissions.get(entity)

        # If permissions map does not specify an action to authorize for entity
        # or if the specified action is `None`, deny access.
        if action is None:
            query = query.filter(expression.false())  # type: ignore
            continue

        assert isinstance(session, Session)
        authorized_filter = authorize_model(oso, user, action, session, entity)
        if authorized_filter is not None:
            query = query.filter(authorized_filter)  # type: ignore

    return query


def authorized_sessionmaker(
    get_oso: Callable[[], Oso],
    get_user: Callable[[], Any],
    get_checked_permissions: Callable[[], Permissions],
    class_: Type[Session] = None,
    **kwargs
):
    """Session factory for sessions with Oso authorization applied.

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

    NOTE: Unless ``enable_baked_queries=True`` is passed as a keyword argument,
          _baked_queries are disabled since the caching mechanism can bypass
          authorization by using queries from the cache that were previously
          baked without authorization applied.

    .. _baked_queries: https://docs.sqlalchemy.org/en/13/orm/extensions/baked.html
    """
    if class_ is None:
        class_ = Session

    # Oso, user, and checked permissions must remain unchanged for the entire
    # session. This is to prevent unauthorized objects from ending up in the
    # session's identity map.
    class Sess(AuthorizedSessionBase, class_):  # type: ignore
        def __init__(self, **options):
            options.setdefault("oso", get_oso())
            options.setdefault("user", get_user())
            options.setdefault("checked_permissions", get_checked_permissions())
            super().__init__(**options)

    session = type("Session", (Sess,), {})

    # We call sessionmaker here because sessionmaker adds a configure
    # method to the returned session and we want to replicate that
    # functionality.
    return sessionmaker(class_=session, **kwargs)


def scoped_session(
    get_oso: Callable[[], Oso],
    get_user: Callable[[], Any],
    get_checked_permissions: Callable[[], Permissions],
    scopefunc: Optional[Callable[..., Any]] = None,
    **kwargs
):
    """Return a scoped session maker that uses the Oso instance, user, and
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

    NOTE: Unless ``enable_baked_queries=True`` is passed as a keyword argument,
          _baked_queries are disabled since the caching mechanism can bypass
          authorization by using queries from the cache that were previously
          baked without authorization applied.

    .. _scoped_session: https://docs.sqlalchemy.org/en/13/orm/contextual.html

    .. _baked_queries: https://docs.sqlalchemy.org/en/13/orm/extensions/baked.html
    """
    scopefunc = scopefunc or (lambda: None)

    def _scopefunc():
        checked_permissions = frozenset(get_checked_permissions().items())
        return (get_oso(), checked_permissions, get_user(), scopefunc())

    factory = authorized_sessionmaker(
        get_oso, get_user, get_checked_permissions, **kwargs
    )

    return orm.scoped_session(factory, scopefunc=_scopefunc)


class AuthorizedSessionBase(object):
    """Mixin for SQLAlchemy Session that uses oso authorization for queries.

    Can be used to create a custom session class that uses oso::

        class MySession(AuthorizedSessionBase, sqlalchemy.orm.Session):
            pass

    NOTE: Unless ``enable_baked_queries=True`` is passed to the constructor,
          _baked_queries are disabled since the caching mechanism can bypass
          authorization by using queries from the cache that were previously
          baked without authorization applied.

    .. _baked_queries: https://docs.sqlalchemy.org/en/13/orm/extensions/baked.html
    """

    def __init__(self, oso: Oso, user, checked_permissions: Permissions, **options):
        """Create an authorized session using ``oso``.

        :param oso: The Oso instance to use for authorization.
        :param user: The user to perform authorization for.
        :param checked_permissions: The permissions (resource-action pairs) to
                                    authorize.
        :param checked_permissions: An optional map of permissions
                                    (resource-action pairs) to authorize for
                                    the session. If ``None`` is provided,
                                    no authorization will be applied to
                                    the session. If a map of permissions
                                    is provided, querying for a SQLAlchemy
                                    model present in the map will authorize
                                    results according to the action
                                    specified as the value in the map. E.g.,
                                    providing a map of ``{Post: "read",
                                    User: "view"}`` where ``Post`` and
                                    ``User`` are SQLAlchemy models will apply
                                    authorization to ``session.query(Post)``
                                    and ``session.query(User)`` such that
                                    only ``Post`` objects that the user can
                                    ``"read"`` and ``User`` objects that the
                                    user can ``"view"`` are fetched from the
                                    database.
        :param options: Additional keyword arguments to pass to ``Session``.

        **Invariant**: the `oso`, `user`, and `checked_permissions` parameters
        *must remain fixed for a given session*. This prevents authorization
        responses from changing, ensuring that the session's identity map never
        contains unauthorized objects.
        """
        self._oso = oso
        self._oso_user = user
        self._oso_checked_permissions = checked_permissions

        # Unless a user explicitly enables baked queries with the understanding
        # that it result in authorization bypasses, disable them.
        if "enable_baked_queries" not in options:
            options["enable_baked_queries"] = False

        super().__init__(**options)  # type: ignore

    @property
    def oso_context(self):
        return {
            "oso": self._oso,
            "user": self._oso_user,
            "checked_permissions": self._oso_checked_permissions,
        }


class AuthorizedSession(AuthorizedSessionBase, Session):
    """SQLAlchemy session that uses oso for authorization.

    Queries on this session only return authorized objects.

    Usually :py:func:`authorized_sessionmaker` is used instead of directly
    instantiating the session.

    NOTE: Unless ``enable_baked_queries=True`` is passed to the constructor,
          _baked_queries are disabled since the caching mechanism can bypass
          authorization by using queries from the cache that were previously
          baked without authorization applied.

    .. _baked_queries: https://docs.sqlalchemy.org/en/13/orm/extensions/baked.html
    """

    pass
