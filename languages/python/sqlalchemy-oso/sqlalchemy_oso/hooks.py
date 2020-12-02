"""SQLAlchemy hooks that transparently enable oso on SQLAlchemy operations.

There are several potential interfaces to integrate oso with SQLAlchemy:

    - :py:func:`authorized_sessionmaker`: (**recommended**) Session factory that
       creates a session that applies authorization on every query.
    - :py:func:`enable_hooks`: Globally enable oso on all queries.
    - :py:func:`make_authorized_query_cls`: Make a query class that is
       authorized before execution.

.. note::

    If using any API besides :py:func:`authorized_sessionmaker`, ensure you set
    ``enable_baked_queries=False`` on the session. Query caching can interfere
    with authorization.

    It is recommended to scope authorization context (the oso instance, user and
    action) to a single session.  Otherwise, the identity map (SQLAlchemy's
    cache of retrieved objects) may contain objects that were authorized for a
    previous user. This could cause incorrect behavior.

    :py:func:`authorized_sessionmaker` will enforce this.  If the authorization
    context changes during the session, an Exception will be raised.
"""
import functools
from typing import Any, Callable

from sqlalchemy.event import listen, remove
from sqlalchemy.orm.query import Query
from sqlalchemy.orm import sessionmaker

from oso import Oso

from sqlalchemy_oso.auth import authorize_model_filter


def enable_hooks(
    get_oso: Callable[[], Oso],
    get_user: Callable[[], Any],
    get_action: Callable[[], Any],
    target=None,
):
    """Enable all SQLAlchemy hooks."""
    if target is None:
        target = Query

    return enable_before_compile(target, get_oso, get_user, get_action)


def enable_before_compile(
    target,
    get_oso: Callable[[], Oso],
    get_user: Callable[[], Any],
    get_action: Callable[[], Any],
):
    """Enable before compile hook."""
    auth = functools.partial(
        authorize_query, get_oso=get_oso, get_user=get_user, get_action=get_action
    )

    listen(target, "before_compile", auth, retval=True)

    return lambda: remove(target, "before_compile", auth)


def authorize_query(query: Query, get_oso, get_user, get_action) -> Query:
    """Authorize an existing query with an oso instance, user and action."""
    oso = get_oso()
    action = get_action()
    actor = get_user()

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

        authorized_filter = authorize_model_filter(
            oso, actor, action, query.session, entity
        )
        if authorized_filter is not None:
            query = query.filter(authorized_filter)

    return query


def make_authorized_query_cls(get_oso, get_user, get_action) -> Query:
    class AuthorizedQuery(Query):
        """Query object that always applies authorization for ORM entities."""

    enable_hooks(get_oso, get_user, get_action, target=AuthorizedQuery)

    return AuthorizedQuery


def authorized_sessionmaker(get_oso, get_user, get_action, *args, **kwargs):
    """Session factory for sessions with oso authorization applied.

    :param get_oso: Callable that return oso instance to use for authorization.
    :param get_user: Callable that returns user for an authorization request.
    :param get_action: Callable that returns action for the authorization request.

    The ``query_cls`` parameter cannot be used with ``authorize_sessionmaker``.

    Baked queries will be disabled for this session, because they are incompatible
    with authorization.

    All other positional and keyword arguments are passed through to
    :py:func:`sqlalchemy.orm.session.sessionmaker` unchanged.
    """
    # TODO (dhatch): Should be possible with additional wrapping.
    assert (
        "query_cls" not in kwargs
    ), "Cannot use custom query class with authorized_sessionmaker."

    # oso = get_oso()
    # user = get_user()
    # action = get_action()

    # oso, user and action must remain unchanged for the entire session.
    # If they change before a query runs, an error is thrown.
    # This is to prevent objects that are unauthorized from ending up in the
    # session's identity map.

    # TODO (dhatch): The scope of these is wrong. They should be run when
    # a session is created, which probably requires a custom session class or
    # more customization of the sessionmaker.
    # def checked_get_oso():
    #     if get_oso() != oso:
    #         # TODO proper error type.
    #         raise Exception("oso object changed during session.")
    #     return oso

    # def checked_get_user():
    #     if get_user() != user:
    #         raise Exception("user object changed during session.")
    #     return user

    # def checked_get_action():
    #     if get_action() != action:
    #         raise Exception("action changed during session.")
    #     return action

    return sessionmaker(
        query_cls=make_authorized_query_cls(get_oso, get_user, get_action),
        *args,
        **kwargs
    )
