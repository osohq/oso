"""SQLAlchemy signal for processing queries."""
from typing import Optional
import logging

from sqlalchemy import event, inspect
from sqlalchemy.orm import Session
from sqlalchemy.orm.util import AliasedClass
from sqlalchemy_oso.session import AuthorizedSessionBase, Permissions
from sqlalchemy.sql import expression as expr

from oso import Oso

from sqlalchemy_oso.auth import authorize_model

logger = logging.getLogger(__name__)

try:
    # TODO(gj): remove type ignore once we upgrade to 1.4-aware MyPy types.
    from sqlalchemy.orm import with_loader_criteria  # type: ignore
    from sqlalchemy_oso.sqlalchemy_utils import all_entities_in_statement

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
        checked_permissions: Permissions = session.oso_context["checked_permissions"]

        # Early return if no authorization is to be applied.
        if checked_permissions is None:
            return

        entities = all_entities_in_statement(execute_state.statement)
        logger.info(f"Authorizing entities: {entities}")
        for entity in entities:
            action = checked_permissions.get(entity)

            # If permissions map does not specify an action to authorize for entity
            # or if the specified action is `None`, deny access.
            if action is None:
                logger.warning(f"No allowed action for entity {entity}")
                where = with_loader_criteria(entity, expr.false(), include_aliases=True)
                execute_state.statement = execute_state.statement.options(where)
            else:
                filter = authorize_model(oso, user, action, session, entity)
                if filter is not None:
                    logger.info(f"Applying filter {filter} to entity {entity}")
                    where = with_loader_criteria(entity, filter, include_aliases=True)
                    execute_state.statement = execute_state.statement.options(where)
                else:
                    logger.warning(f"Policy did not return filter for entity {entity}")


except ImportError:
    from sqlalchemy.orm.query import Query

    @event.listens_for(Query, "before_compile", retval=True)
    def _before_compile(query):
        """Enable before compile hook."""
        return _authorize_query(query)

    def _authorize_query(query: Query) -> Optional[Query]:
        """Authorize an existing query with an Oso instance, user, and a
        permissions map indicating which actions to check for which SQLAlchemy
        models."""
        session = query.session

        # Early return if this isn't an authorized session.
        if not isinstance(session, AuthorizedSessionBase):
            return None

        oso: Oso = session.oso_context["oso"]
        user = session.oso_context["user"]
        checked_permissions: Permissions = session.oso_context["checked_permissions"]

        # Early return if no authorization is to be applied.
        if checked_permissions is None:
            return None

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

            # If entity is an alias, get the action for the underlying class.
            if isinstance(entity, AliasedClass):
                action = checked_permissions.get(inspect(entity).class_)  # type: ignore
            else:
                action = checked_permissions.get(entity)

            # If permissions map does not specify an action to authorize for entity
            # or if the specified action is `None`, deny access.
            if action is None:
                query = query.filter(expr.false())  # type: ignore
                continue

            assert isinstance(session, Session)
            authorized_filter = authorize_model(oso, user, action, session, entity)
            if authorized_filter is not None:
                query = query.filter(authorized_filter)  # type: ignore

        return query
