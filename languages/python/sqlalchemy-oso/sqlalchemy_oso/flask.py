try:
    from flask import _app_ctx_stack
    from flask_sqlalchemy import SQLAlchemy, SignallingSession
except ImportError:
    import warnings
    warnings.warn("Missing depenedencies for Flask. Install sqlalchemy-oso with the flask extra.")
    raise

from sqlalchemy_oso.hooks import authorized_sessionmaker, scoped_session

class AuthorizedSQLAlchemy(SQLAlchemy):
    """flask_sqlalchemy ``SQLAlchemy`` subclass that uses oso.

    Creates sessions with oso authorization applied.
    """
    def __init__(self,
                 get_oso,
                 get_user,
                 get_action,
                 **kwargs):
        self._get_oso = get_oso
        self._get_user = get_user
        self._get_action = get_action
        super().__init__(**kwargs)

    def create_session(self, options):
        return authorized_sessionmaker(
            get_oso=self._get_oso,
            get_user=self._get_user,
            get_action=self._get_action,
            **options)

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
            **options)
