"""SQLAlchemy session classes and factories for oso."""
from typing import Any, Callable, Optional, Type

from sqlalchemy import event
from sqlalchemy.orm.query import Query
from sqlalchemy.orm import sessionmaker, Session
from sqlalchemy import orm

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


def _authorize_query(query: Query) -> Optional[Query]:
    """Authorize an existing query with an Oso instance, user, and actions."""
    # Get the query session.
    session = query.session

    # Check whether this is an oso session.
    if not isinstance(session, AuthorizedSessionBase):
        # Not an authorized session.
        return None

    actions: Optional[Any] = session.oso_context["actions"]

    # Early return if no authorization is to be applied.
    if actions is None:
        return None

    oso = session.oso_context["oso"]
    user = session.oso_context["user"]

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

        if isinstance(actions, dict):
            if not entity in actions:
                continue
            action = actions[entity]
        else:
            action = actions

        session = query.session
        assert isinstance(session, Session)
        authorized_filter = authorize_model(oso, user, action, session, entity)
        if authorized_filter is not None:
            query = query.filter(authorized_filter)  # type: ignore

    return query


def authorized_sessionmaker(
    get_oso: Callable[[], Oso],
    get_user: Callable[[], Any],
    get_actions: Callable[[], Any],
    class_: Type[Session] = None,
    **kwargs
):
    """Session factory for sessions with Oso authorization applied.

    :param get_oso: Callable that returns the Oso instance to use for
                    authorization.
    :param get_user: Callable that returns the user for an authorization
                     request.
    :param get_actions: Callable that returns the actions to authorize for the
                     request.
    :param class_: Base class to use for sessions.

    All other keyword arguments are passed through to
    :py:func:`sqlalchemy.orm.session.sessionmaker` unchanged.
    """
    if class_ is None:
        class_ = Session

    # Oso, user, and actions must remain unchanged for the entire session. This
    # is to prevent unauthorized objects from ending up in the session's
    # identity map.
    class Sess(AuthorizedSessionBase, class_):
        def __init__(self, **options):
            options.setdefault("oso", get_oso())
            options.setdefault("user", get_user())
            options.setdefault("actions", get_actions())
            super().__init__(**options)

    session = type("Session", (Sess,), {})

    # We call sessionmaker here because sessionmaker adds a configure
    # method to the returned session and we want to replicate that
    # functionality.
    return sessionmaker(class_=session, **kwargs)


def scoped_session(get_oso, get_user, get_actions, scopefunc=None, **kwargs):
    """Return a scoped session maker that uses the user and actions as part of
    the scope function.

    Use in place of sqlalchemy's scoped_session_.

    Uses :py:func:`authorized_sessionmaker` as the factory.

    :param get_oso: Callable that returns the Oso instance to use for
                    authorization.
    :param get_user: Callable that returns the user for an authorization
                     request.
    :param get_actions: Callable that returns the actions to authorize for the
                     request.
    :param scopefunc: Additional scope function to use for scoping sessions.
                      Output will be combined with the Oso, actions, and user
                      objects.
    :param kwargs: Additional keyword arguments to pass to
                   :py:func:`authorized_sessionmaker`.

    .. _scoped_session: https://docs.sqlalchemy.org/en/13/orm/contextual.html
    """
    scopefunc = scopefunc or (lambda: None)

    def _scopefunc():
        return (get_oso(), get_actions(), get_user(), scopefunc())

    factory = authorized_sessionmaker(get_oso, get_user, get_actions, **kwargs)

    return orm.scoped_session(factory, scopefunc=_scopefunc)


class AuthorizedSessionBase(object):
    """Mixin for SQLAlchemy Session that uses oso authorization for queries.

    Can be used to create a custom session class that uses oso::

        class MySession(AuthorizedSessionBase, sqlalchemy.orm.Session):
            pass
    """

    def __init__(self, oso: Oso, user, actions, **options):
        """Create an authorized session using ``oso``.

        :param oso: The Oso instance to use for authorization.
        :param user: The user to perform authorization for.
        :param actions: The actions to authorize.
        :param options: Additional keyword arguments to pass to ``Session``.

        The user and actions parameters are fixed for a given session. This
        prevents authorization responses from changing, ensuring that the
        identity map never contains unauthorized objects.
        """
        self._oso = oso
        self._oso_user = user
        self._oso_actions = actions

        super().__init__(**options)  # type: ignore

    @property
    def oso_context(self):
        return {"oso": self._oso, "user": self._oso_user, "actions": self._oso_actions}


class AuthorizedSession(AuthorizedSessionBase, Session):
    """SQLAlchemy session that uses oso for authorization.

    Queries on this session only return authorized objects.

    Usually :py:func:`authorized_sessionmaker` is used instead of directly
    instantiating the session.
    """

    pass
