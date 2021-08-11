from oso import Oso
from polar import Variable
from polar.exceptions import PolarRuntimeError
from polar.partial import TypeConstraint

from sqlalchemy.orm.query import Query
from sqlalchemy.orm.session import Session
from sqlalchemy import inspect
from sqlalchemy.sql import expression as sql

from sqlalchemy_oso.partial import partial_to_filter
from sqlalchemy_oso import roles

from sqlalchemy_oso.compat import iterate_model_classes

from functools import reduce


def polar_model_name(model) -> str:
    """Return polar class name for SQLAlchemy model."""
    return model.__name__


def null_query(session: Session, model) -> Query:
    """Return an intentionally empty query."""
    # TODO (dhatch): Make this not hit the database.
    return session.query(model).filter(sql.false())


def register_models(oso: Oso, base_or_registry):
    """Register all models in registry (SQLAlchemy 1.4) or declarative base
    class (1.3 and 1.4) ``base_or_registry`` with Oso as classes."""
    for model in iterate_model_classes(base_or_registry):
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

    def get_field_type(model, field):
        try:
            field = getattr(model, field)
        except AttributeError:
            raise PolarRuntimeError(f"Cannot get property {field} on {model}.")

        try:
            return field.entity.class_
        except AttributeError as e:
            raise PolarRuntimeError(
                f"Cannot determine type of {field} on {model}."
            ) from e

    oso.host.get_field = get_field_type

    try:
        mapped_class = inspect(model, raiseerr=True).class_
    except AttributeError:
        raise TypeError(f"Expected a model; received: {model}")

    resource = Variable("resource")
    constraint = TypeConstraint(resource, polar_model_name(mapped_class))
    results = oso.query_rule(
        "allow",
        actor,
        action,
        resource,
        bindings={resource: constraint},
        accept_expression=True,
    )

    combined_filter = None
    has_result = False
    for result in results:
        has_result = True

        resource_partial = result["bindings"]["resource"]
        if isinstance(resource_partial, model):

            def f(pk):
                return getattr(model, pk) == getattr(resource_partial, pk)

            filters = [f(pk.name) for pk in inspect(model).primary_key]
            filter = reduce(lambda a, b: a & b, filters)

        else:
            filter, role_method = partial_to_filter(
                resource_partial, session, model, get_model=oso.get_class
            )

            if role_method is not None:
                roles_filter = roles._generate_query_filter(oso, role_method, model)
                filter &= roles_filter

        if combined_filter is None:
            combined_filter = filter
        else:
            combined_filter = combined_filter | filter

    if not has_result:
        return sql.false()

    return combined_filter
