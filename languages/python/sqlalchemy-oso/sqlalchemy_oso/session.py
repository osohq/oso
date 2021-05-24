"""SQLAlchemy session classes and factories for oso."""
from typing import Optional

from sqlalchemy import event
from sqlalchemy.orm import sessionmaker, Session
from sqlalchemy import orm

from oso import Oso

from sqlalchemy_oso.auth import authorize_model
from sqlalchemy_oso.compat import USING_SQLAlchemy_v1_3


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


def authorized_sessionmaker(get_oso, get_user, get_action, class_=None, **kwargs):
    """Session factory for sessions with oso authorization applied.

    :param get_oso: Callable that return oso instance to use for authorization.
    :param get_user: Callable that returns user for an authorization request.
    :param get_action: Callable that returns action for the authorization request.
    :param class_: Base class to use for sessions.

    All other keyword arguments are passed through to
    :py:func:`sqlalchemy.orm.session.sessionmaker` unchanged.

    NOTE: _baked_queries are disabled on SQLAlchemy 1.3 since the caching
          mechanism can bypass authorization by using queries from the cache
          that were previously baked without authorization applied. Note that
          _baked_queries are deprecated as of SQLAlchemy 1.4.

    .. _baked_queries: https://docs.sqlalchemy.org/en/14/orm/extensions/baked.html
    """
    if class_ is None:
        class_ = Session

    # oso, user and action must remain unchanged for the entire session.
    # This is to prevent objects that are unauthorized from ending up in the
    # session's identity map.
    class Sess(AuthorizedSessionBase, class_):
        def __init__(self, **options):
            options.setdefault("oso", get_oso())
            options.setdefault("user", get_user())
            options.setdefault("action", get_action())
            super().__init__(**options)

    session = type("Session", (Sess,), {})

    # We call sessionmaker here because sessionmaker adds a configure
    # method to the returned session and we want to replicate that
    # functionality.
    return sessionmaker(class_=session, **kwargs)


def scoped_session(get_oso, get_user, get_action, scopefunc=None, **kwargs):
    """Return a scoped session maker that uses the user and action as part of the scope function.

    Use in place of sqlalchemy's scoped_session_

    Uses :py:func:`authorized_sessionmaker` as the factory.

    :param get_oso: Callable that return oso instance to use for authorization.
    :param get_user: Callable that returns user for an authorization request.
    :param get_action: Callable that returns action for the authorization request.
    :param scopefunc: Additional scope function to use for scoping sessions.
                      Output will be combined with the oso, action and user objects.
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
        return (get_oso(), get_action(), get_user(), scopefunc())

    factory = authorized_sessionmaker(get_oso, get_user, get_action, **kwargs)

    return orm.scoped_session(factory, scopefunc=_scopefunc)


class AuthorizedSessionBase(object):
    """Mixin for SQLAlchemy Session that uses oso authorization for queries.

    Can be used to create a custom session class that uses oso::

        class MySession(AuthorizedSessionBase, sqlalchemy.orm.Session):
            pass

    NOTE: _baked_queries are disabled on SQLAlchemy 1.3 since the caching
          mechanism can bypass authorization by using queries from the cache
          that were previously baked without authorization applied. Note that
          _baked_queries are deprecated as of SQLAlchemy 1.4.

    .. _baked_queries: https://docs.sqlalchemy.org/en/14/orm/extensions/baked.html
    """

    def __init__(self, oso: Oso, user, action, **options):
        """Create an authorized session using ``oso``.

        :param oso: The oso instance to use for authorization.
        :param user: The user to perform authorization for.
        :param action: The action to authorize.
        :param options: Additional keyword arguments to pass to ``Session``.

        The user and action parameters are fixed for a given session. This
        prevents authorization responses from changing, ensuring that the
        identity map never contains unauthorized objects.
        """
        self._oso = oso
        self._oso_user = user
        self._oso_action = action

        if USING_SQLAlchemy_v1_3:  # Disable baked queries on SQLAlchemy 1.3.
            options["enable_baked_queries"] = False

        super().__init__(**options)  # type: ignore

    @property
    def oso_context(self):
        return {"oso": self._oso, "user": self._oso_user, "action": self._oso_action}


class AuthorizedSession(AuthorizedSessionBase, Session):
    """SQLAlchemy session that uses oso for authorization.

    Queries on this session only return authorized objects.

    Usually :py:func:`authorized_sessionmaker` is used instead of directly
    instantiating the session.

    NOTE: _baked_queries are disabled on SQLAlchemy 1.3 since the caching
          mechanism can bypass authorization by using queries from the cache
          that were previously baked without authorization applied. Note that
          _baked_queries are deprecated as of SQLAlchemy 1.4.

    .. _baked_queries: https://docs.sqlalchemy.org/en/14/orm/extensions/baked.html
    """

    pass


try:
    # TODO(gj): remove type ignore once we upgrade to 1.4-aware MyPy types.
    from sqlalchemy.orm import with_loader_criteria  # type: ignore

    @event.listens_for(Session, "do_orm_execute")
    def do_orm_execute(execute_state):
        if not execute_state.is_select:
            return

        session = execute_state.session

        if not isinstance(session, AuthorizedSessionBase):
            return
        assert isinstance(session, Session)

        oso: Oso = session.oso_context["oso"]
        user = session.oso_context["user"]
        action = session.oso_context["action"]

        def entities_in_statement(statement):
            def _entities_in_statement(statement):
                try:
                    entities = (cd["entity"] for cd in statement.column_descriptions)
                    return set(e for e in entities if e is not None)
                except AttributeError:
                    return set()

            entities = _entities_in_statement(statement)

            # TODO(gj): currently walking way more than we have to. Probably
            # some points in the tree where we can safely call it good for that
            # branch and continue on to more fruitful pastures.
            for child in statement.get_children():
                entities |= entities_in_statement(child)

            return entities

        for entity in entities_in_statement(execute_state.statement):
            filter = authorize_model(oso, user, action, session, entity)
            if filter is not None:
                where = with_loader_criteria(entity, filter, include_aliases=True)
                execute_state.statement = execute_state.statement.options(where)


except ImportError:
    from sqlalchemy.orm.query import Query

    @event.listens_for(Query, "before_compile", retval=True)
    def _before_compile(query):
        """Enable before compile hook."""
        return _authorize_query(query)

    def _authorize_query(query: Query) -> Optional[Query]:
        """Authorize an existing query with an oso instance, user and action."""
        # Get the query session.
        session = query.session

        # Check whether this is an oso session.
        if not isinstance(session, AuthorizedSessionBase):
            # Not an authorized session.
            return None

        oso = session.oso_context["oso"]
        user = session.oso_context["user"]
        action = session.oso_context["action"]

        # TODO (dhatch): This is necessary to allow ``authorize_query`` to work
        # on queries that have already been made.  If a query has a LIMIT or OFFSET
        # applied, SQLAlchemy will by default throw an error if filters are applied.
        # This prevents these errors from occuring, but could result in some
        # incorrect queries. We should remove this if possible.
        query = query.enable_assertions(False)

        entities = {column["entity"] for column in query.column_descriptions}
        for entity in entities:
            # Only apply authorization to columns that represent a mapper entity.
            if entity is None:
                continue

            authorized_filter = authorize_model(
                oso, user, action, query.session, entity
            )
            if authorized_filter is not None:
                query = query.filter(authorized_filter)

        return query
