from oso import Enforcer
from .oso import SQLAlchemyPolicy
from .auth import authorize_model


class SQLAlchemyEnforcer(Enforcer):
    """
    NOTE: This is a preview feature.

    Custom Oso enforcer for SQLAlchemy.
    """

    def __init__(self, policy: SQLAlchemyPolicy, session_maker, *args, **kwargs):
        """Construct a new SQLAlchemyEnforcer

        >>> policy = SQLAlchemyPolicy(Base)
        >>> oso = SQLAlchemyEnforcer(policy, Session)
        >>> # ...
        >>> oso.authorize_query(user, Article).all()

        :param policy: An instance of ``SQLAlchemyPolicy``
        :param session_maker: A SQLAlchemy session maker instance"""
        self.session_maker = session_maker
        super().__init__(policy, *args, **kwargs)

    def authorize_query(self, actor, model, action=None):
        """Authorize a model query, returning a SQLAlchemy ``Query`` instance.

        Uses the ``allow`` rule to determine which constraints to apply. The
        query instance returned contains only results that the actor can
        ``"read"``.

        :param actor: The current actor
        :param model: A SQLAlchemy model class
        :param action: Optionally override the action used to filter results.
        Defaults to the ``read_action`` of this enforcer, which is normally the
        string ``"read"``.
        """
        if action is None:
            action = self.read_action

        session = self.session_maker()
        filter = authorize_model(
            oso=self.policy,
            actor=actor,
            action=self.read_action,
            session=session,
            model=model,
        )
        return session.query(model).filter(filter)
