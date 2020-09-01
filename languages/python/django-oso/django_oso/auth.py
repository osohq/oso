from django.core.exceptions import PermissionDenied

from .oso import Oso

def authorize(request, resource, *, actor=None, action=None):
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
    """
    _set_request_authorized(request)

def request_authorized(request) -> bool:
    """Return ``true`` if ``request`` was authorized using :py:func:`authorize`."""
    return getattr(request, "_oso_authorized", False)

def _set_request_authorized(request):
    """Mark request as being authorized."""
    request._oso_authorized = True
