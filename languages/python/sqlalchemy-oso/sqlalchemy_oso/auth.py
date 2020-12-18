from oso import Oso
from polar.partial import Partial, TypeConstraint

from sqlalchemy.orm.query import Query
from sqlalchemy.orm.session import Session
from sqlalchemy import inspect
from sqlalchemy.sql import expression as sql

from sqlalchemy_oso.partial import partial_to_filter


def polar_model_name(model) -> str:
    """Return polar class name for SQLAlchemy model."""
    return model.__name__


def null_query(session: Session, model) -> Query:
    """Return an intentionally empty query."""
    # TODO (dhatch): Make this not hit the database.
    return session.query(model).filter(sql.false())


def register_models(oso: Oso, base):
    """Register all models in model base class ``base`` with oso as classes."""
    # TODO (dhatch): Not sure this is legit b/c it uses an internal interface?
    for name, model in base._decl_class_registry.items():
        if name == "_sa_module_registry":
            continue

        oso.register_class(model)


def authorize_model(oso: Oso, actor, action, session: Session, model):
    """Return SQLAlchemy expression that applies the policy to ``model``.

    Executing this query will return only authorized objects. If the request is
    not authorized, a query that always contains no result will be returned.

    :param oso: The oso class to use for evaluating the policy.
    :param actor: The actor to authorize.
    :param action: The action to authorize.

    :param session: The SQLAlchemy session.
    :param model: The model to authorize, must be a SQLAlchemy model or alias.
    """
    try:
        mapped_class = inspect(model, raiseerr=True).class_
    except AttributeError:
        raise TypeError(f"Expected a model; received: {model}")

    partial_resource = Partial(
        "resource", TypeConstraint(polar_model_name(mapped_class))
    )
    results = oso.query_rule("allow", actor, action, partial_resource)

    combined_filter = None
    has_result = False
    for result in results:
        has_result = True

        resource_partial = result["bindings"]["resource"]
        filter = partial_to_filter(
            resource_partial, session, model, get_model=oso.get_class
        )
        if combined_filter is None:
            combined_filter = filter
        else:
            combined_filter = combined_filter | filter

    if not has_result:
        return sql.false()

    return combined_filter
