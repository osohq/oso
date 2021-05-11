from oso import Oso, OsoError
from .auth import register_models
from .roles2 import OsoRoles


class SQLAlchemyOso(Oso):
    """The central object to manage application policy state, e.g.
    the policy data, and verify requests when using Oso with SQLAlchemy.

    Supports SQLAlchemy-specific functionality, including data filtering and role management.

    Accepts a SQLAlchemy declarative_base on initialization, which is used to register
    all relevant SQLAlchemy models with Oso.

    >>> from sqlalchemy_oso import SQLAlchemyOso
    >>> from sqlalchemy.ext.declarative import declarative_base
    >>> Base = declarative_base(name="MyBaseModel")
    >>> SQLAlchemyOso(Base)
    <sqlalchemy_oso.oso.SQLAlchemyOso object at 0x...>

    """

    def __init__(self, sqlalchemy_base):
        super().__init__()

        # Register all sqlalchemy models on sqlalchemy_base
        # TODO (dhatch): Not sure this is legit b/c it uses an internal interface?
        register_models(self, sqlalchemy_base)

        self.base = sqlalchemy_base
        self._roles_enabled = False

    def enable_roles(self, user_model, session_maker):
        """Enable the Oso role management API.
        Oso will create SQLAlchemy models to create and assign roles to users (stored in `user_model`).
        The roles API methods will be available on the `roles` property of `SQLAlchemyOso`.
        """
        self._roles = OsoRoles(
            oso=self,
            sqlalchemy_base=self.base,
            user_model=user_model,
            session_maker=session_maker,
        )
        self._roles_enabled = True

    @property
    def roles(self):
        """Property to access the Oso roles API methods defined in `OsoRoles`.
        This property is only available after calling `enable_roles()`.
        """
        if not self._roles_enabled:
            raise OsoError(
                "Cannot access 'roles' on 'SQLAlchemyOso' before calling 'enable_roles()'"
            )
        return self._roles
