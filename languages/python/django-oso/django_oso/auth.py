from django.core.exceptions import PermissionDenied
from django.db.models import Q, Model
from polar.exceptions import UnsupportedError

from .oso import Oso, polar_model_name
from polar.partial import TypeConstraint
from polar.variable import Variable

from .partial import TRUE_FILTER, partial_to_query_filter


def authorize(request, resource, *, actor=None, action=None):
    """Authorize ``request`` for ``resource``, ``actor`` and ``action``.

    Calls :py:meth:`oso.Oso.is_allowed` with the corresponding arguments. If
    authorization fails, raises a :py:class:`django.core.exceptions.PermissionDenied`
    exception.

    :param actor: The actor making the request. Defaults to ``request.user``.
    :param action: The action to authorize the actor to perform. Defaults to
                    ``request.method``.
    :param resource: The resource to authorize the actor to access.

    :raises django.core.exceptions.PermissionDenied: If the request is not authorized.

    See :py:func:`django_oso.decorators.authorize` for view decorator version of
    this function.
    """
    if actor is None:
        actor = request.user

    if action is None:
        action = request.method

    authorized = Oso.is_allowed(actor, action, resource)
    _set_request_authorized(request)

    if not authorized:
        raise PermissionDenied()


def authorize_model(request, model, *, actor=None, action=None) -> Q:
    """Authorize ``request`` for django model ``model``, ``actor``, and ``action``.

    .. warning::

        This feature is currently in preview.

    Partially evaluates the Polar rule ``allow(actor, action, Variable(model))``. If
    authorization fails, raises a :py:class:`django.core.exceptions.PermissionDenied`
    exception.

    Otherwise, returns a django ``Q`` object representing a filter that must be
    applied to ``model``. This object can be applied to filter query results to
    only contain authorized objects.

    For example::

        post_filter = authorize_model(request, Post)
        authorized_posts = Post.objects.filter(post_filter)

    See also:

    - :py:class:`django_oso.models.AuthorizedModel`

    :param actor: The actor making the request. Defaults to ``request.user``.
    :param action: The action to authorize the actor to perform. Defaults to
                    ``request.method``.
    :param model: The model to authorize access for.

    :raises django.core.exceptions.PermissionDenied: If the request is not authorized.
    :returns: A django ``Q`` object representing the authorization filter.
    """
    if Oso._polar_roles_enabled:
        raise UnsupportedError(
            "Data filtering not yet supported with Polar roles enabled."
        )

    if actor is None:
        actor = request.user

    if action is None:
        action = request.method

    assert issubclass(model, Model), f"Expected a model; received: {model}"
    resource = Variable("resource")
    constraint = TypeConstraint(resource, polar_model_name(model))
    results = Oso.query_rule(
        "allow",
        actor,
        action,
        resource,
        bindings={resource: constraint},
        accept_expression=True,
    )

    filter = None
    for result in results:
        resource_partial = result["bindings"]["resource"]
        if filter is None:
            filter = Q()

        if isinstance(resource_partial, model):
            next_filter = Q(pk=resource_partial.pk)
        else:
            next_filter = partial_to_query_filter(resource_partial, model)

        if next_filter == TRUE_FILTER:
            return TRUE_FILTER

        filter |= next_filter

    if filter is None:
        raise PermissionDenied()

    return filter


def skip_authorization(request):
    """Mark ``request`` as not requiring authorization.

    Use with the :py:func:`django_oso.middleware.RequireAuthorization`
    middleware to silence missing authorization errors.

    See :py:func:`django_oso.decorators.skip_authorization` for view decorator
    version of this function.
    """
    _set_request_authorized(request)


def request_authorized(request) -> bool:
    """Return ``true`` if ``request`` was authorized using :py:func:`authorize`."""
    return getattr(request, "_oso_authorized", False)


def _set_request_authorized(request):
    """Mark request as being authorized."""
    request._oso_authorized = True
