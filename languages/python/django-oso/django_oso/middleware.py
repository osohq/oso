"""Middleware"""

from oso import OsoError

from .auth import request_authorized, authorize

# TODO (dhatch): Make this configurable.
# HTTP status codes that are permitted without authorization.
WHITELIST_STATUSES_DEFAULT = {
    401,
    403,
    404,
    405,
    500,
}


def RequireAuthorization(get_response):
    """Check that ``authorize`` was called during the request.

    :raises oso.OsoError: If ``authorize`` was not called during request
                              processing.

    .. warning::

        This check is performed at the end of request processing before
        returning a response.  If any database modifications are committed
        during the request, but it was not authorized, an OsoError will be
        raised, but the database modifications will not be rolled back.

        .. todo:: Would be good to have a solution to this ^, maybe a on
                  precommit hook.
    """

    def middleware(request):
        response = get_response(request)
        if response.status_code in WHITELIST_STATUSES_DEFAULT:
            return response

        # Ensure authorization occurred.
        if not request_authorized(request):
            raise OsoError("authorize was not called during processing request.")

        return response

    return middleware


def RouteAuthorization(get_response):
    """Perform route authorization on every request.

    A call to
    :py:meth:`~django_oso.auth.authorize`
    will be made before view functions are called with the parameters
    ``actor=request.user, action=request.method, resource=request``.

    Rules in oso policies can be written over requests using the ``HttpRequest``
    specializer:

    .. code-block:: polar

        allow(actor, action, resource: HttpRequest) if
            # Access request properties to perform authorization
            request.path = "/";

    .. note::

        If the view returns a 4**, or 5** HTTP status, this will be returned to
        the end user even if authorization was not performed.

        .. todo:: Customize this ^
    """

    def middleware(request):
        authorize(request, resource=request)
        return get_response(request)

    return middleware
