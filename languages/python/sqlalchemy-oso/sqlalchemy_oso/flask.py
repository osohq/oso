"""Wrappers for using ``sqlalchemy_oso`` with the flask_sqlalchemy_ library.

.. _flask_sqlalchemy: https://flask-sqlalchemy.palletsprojects.com/en/2.x/
"""

try:
    from flask_sqlalchemy import SignallingSession, SQLAlchemy
    from flask_sqlalchemy import __version__ as fv  # type: ignore
    from packaging.version import parse

    if parse(fv) >= parse("3.0"):
        import warnings

        warnings.warn(
            "Flask-SQLAlchemy versions greater than 2.x are not supported. More info: https://github.com/osohq/oso/issues/1631"
        )
        raise
except ImportError:
    import warnings

    warnings.warn(
        "Missing dependencies for Flask. Install sqlalchemy-oso with the flask extra."
    )
    raise

try:
    from greenlet import getcurrent as _get_ident  # type: ignore
except ImportError:
    from threading import get_ident as _get_ident  # type: ignore

from typing import Any, Callable, Mapping, MutableMapping, Optional

import sqlalchemy.orm
from oso import Oso

from sqlalchemy_oso.session import authorized_sessionmaker, scoped_session

from .session import Permissions


class AuthorizedSQLAlchemy(SQLAlchemy):
    """flask_sqlalchemy ``SQLAlchemy`` subclass that uses oso.

    Creates sessions with oso authorization applied. See flask_sqlalchemy_ documentation
    for more information on using flask_sqlalchemy.

    :param get_oso: Callable that returns the :py:class:`oso.Oso` instance to use for authorization.
    :param get_user: Callable that returns the user to authorize for the current request.
    :param get_checked_permissions: Callable that returns the permissions to authorize for the current request.

    >>> from sqlalchemy_oso.flask import AuthorizedSQLAlchemy
    >>> db = AuthorizedSQLAlchemy(
    ...    get_oso=lambda: flask.current_app.oso,
    ...    get_user=lambda: flask_login.current_user,
    ...    get_checked_permissions=lambda: {Post: flask.request.method}
    ... )

    .. _flask_sqlalchemy: https://flask-sqlalchemy.palletsprojects.com/en/2.x/
    """

    def __init__(
        self,
        get_oso: Callable[[], Oso],
        get_user: Callable[[], object],
        get_checked_permissions: Callable[[], Permissions],
        **kwargs: Any
    ) -> None:
        self._get_oso = get_oso
        self._get_user = get_user
        self._get_checked_permissions = get_checked_permissions
        super().__init__(**kwargs)

    def create_scoped_session(
        self, options: Optional[MutableMapping[str, Any]] = None
    ) -> sqlalchemy.orm.scoped_session:
        if options is None:
            options = {}

        scopefunc = options.pop("scopefunc", _get_ident)
        return scoped_session(
            get_oso=self._get_oso,
            get_user=self._get_user,
            get_checked_permissions=self._get_checked_permissions,
            scopefunc=scopefunc,
            class_=SignallingSession,
            db=self,
            **options
        )

    def create_session(self, options: Mapping[str, Any]) -> sqlalchemy.orm.sessionmaker:
        return authorized_sessionmaker(
            get_oso=self._get_oso,
            get_user=self._get_user,
            get_checked_permissions=self._get_checked_permissions,
            class_=SignallingSession,
            db=self,
            **options
        )
