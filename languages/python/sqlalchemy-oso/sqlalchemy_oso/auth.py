from oso import Oso
from polar.partial import Partial, TypeConstraint

from sqlalchemy.orm.query import Query
from sqlalchemy.orm.session import Session
from sqlalchemy.sql import expression as sql

from sqlalchemy_oso.partial import partial_to_query

def null_query(session: Session) -> Query:
    """Return an intentionally empty query."""
    # TODO (dhatch): Make this not hit the database.
    return session.query(sql.false()).filter(sql.false())


def register_models(oso: Oso, base):
    """Register all models in model base class ``base`` with oso as classes."""
    # TODO (dhatch): Not sure this is legit b/c it uses an internal interface?
    for name, model in base._decl_class_registry.items():
        if name == '_sa_module_registry':
            continue

        oso.register_class(model)

def authorize_model(oso: Oso, actor, action, session: Session, model) -> Query:
    """Return a query containing filters that apply the policy to ``model``.

    Executing this query will return only authorized objects. If the request is
    not authorized, a query that always contains no result will be returned.

    :param oso: The oso class to use for evaluating the policy.
    :param actor: The actor to authorize.
    :param action: The action to authorize.

    :param session: The SQLAlchemy session.
    :param model: The model to authorize, must be a SQLAlchemy model.
    """
    # TODO (dhatch): Check that model is a model.
    # TODO (dhatch): More robust name mapping?

    partial_resource = Partial("resource", TypeConstraint(model.__name__))
    results = oso.query_rule("allow", actor, action, partial_resource)

    combined_query = None
    for result in results:
        resource_partial = result["bindings"]["resource"]
        query = partial_to_query(resource_partial, session, model)
        if combined_query is None:
            combined_query = query
        else:
            # TODO (dhatch): Use OR here?
            combined_query = combined_query.union(query)

        # if query is empty?

        # if query is everything?

        # otherwise

    if combined_query is None:
        return null_query(session)

    return combined_query
