from oso import Oso

from .auth import register_models


class SQLAlchemyOso(Oso):
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests when using Oso with SQLAlchemy.

    Supports SQLAlchemy-specific functionality, including data filtering.

    Accepts a SQLAlchemy declarative_base on initialization, which is used to register
    all relevant SQLAlchemy models with Oso.

    >>> from sqlalchemy_oso import SQLAlchemyOso
    >>> from sqlalchemy.ext.declarative import declarative_base
    >>> Base = declarative_base(name="MyBaseModel")
    >>> SQLAlchemyOso(Base)
    <sqlalchemy_oso.oso.SQLAlchemyOso object at 0x...>

    """

    def __init__(self, sqlalchemy_base: type) -> None:
        super().__init__()

        # Register all sqlalchemy models on sqlalchemy_base
        register_models(self, sqlalchemy_base)

        self.base = sqlalchemy_base
