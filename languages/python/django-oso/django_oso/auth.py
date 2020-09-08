from django.core.exceptions import PermissionDenied

from .oso import Oso


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
