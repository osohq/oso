__version__ = "0.24.0"

from .auth import register_models
from .oso import SQLAlchemyOso
from .session import authorized_sessionmaker, scoped_session
from .compat import USING_SQLAlchemy_v1_4
from .signal import do_orm_execute

__all__ = [
    "register_models",
    "authorized_sessionmaker",
    "scoped_session",
    "SQLAlchemyOso",
]

try:
    # Only load AsyncIO support is using SQLAlchemy => 1.4
    if not USING_SQLAlchemy_v1_4:
        raise ImportError

    from .async_session import async_scoped_session, async_authorized_sessionmaker

    __all__ += [
        "async_scoped_session",
        "async_authorized_sessionmaker"
    ]
except (ImportError, SyntaxError):
    pass
