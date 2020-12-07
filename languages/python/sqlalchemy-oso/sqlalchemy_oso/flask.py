"""Wrappers for using ``sqlalchemy_oso`` with the flask_sqlalchemy_ library.

.. _flask_sqlalchemy: https://flask-sqlalchemy.palletsprojects.com/en/2.x/
"""

try:
    from flask import _app_ctx_stack  # type: ignore
    from flask_sqlalchemy import SQLAlchemy, SignallingSession
except ImportError:
    import warnings

    warnings.warn(
        "Missing depenedencies for Flask. Install sqlalchemy-oso with the flask extra."
    )
    raise

from sqlalchemy_oso.session import authorized_sessionmaker, scoped_session


class AuthorizedSQLAlchemy(SQLAlchemy):
    """flask_sqlalchemy ``SQLAlchemy`` subclass that uses oso.

    Creates sessions with oso authorization applied. See flask_sqlalchemy_ documentation
    for more information on using flask_sqlalchemy.

    :param get_oso: Callable that returns the :py:class:`oso.Oso` instance to use for authorization.
    :param get_user: Callable that returns the user to authorize for the current request.
    :param get_action: Callable that returns the action to authorize for the current request.

    >>> from sqlalchemy_oso.flask import AuthorizedSQLAlchemy
    >>> db = AuthorizedSQLAlchemy(
    ...    get_oso=lambda: flask.current_app.oso,
    ...    get_user=lambda: flask_login.current_user,
    ...    get_action=lambda: flask.request.method
    ... )

    .. _flask_sqlalchemy: https://flask-sqlalchemy.palletsprojects.com/en/2.x/
    """

    def __init__(self, get_oso, get_user, get_action, **kwargs):
        self._get_oso = get_oso
        self._get_user = get_user
        self._get_action = get_action
        super().__init__(**kwargs)

    def create_scoped_session(self, options=None):
        if options is None:
            options = {}

        scopefunc = options.pop("scopefunc", _app_ctx_stack.__ident_func__)
        return scoped_session(
            get_oso=self._get_oso,
            get_user=self._get_user,
            get_action=self._get_action,
            scopefunc=scopefunc,
            class_=SignallingSession,
            db=self,
            **options
        )

    def create_session(self, options):
        return authorized_sessionmaker(
            get_oso=self._get_oso,
            get_user=self._get_user,
            get_action=self._get_action,
            class_=SignallingSession,
            db=self,
            **options
        )
