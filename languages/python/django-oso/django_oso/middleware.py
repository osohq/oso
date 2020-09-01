"""Middleware"""

from oso import OsoException

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
    """Check that ``authorize`` was called during the request."""
    def middleware(request):
        response = get_response(request)
        if response.status_code in WHITELIST_STATUSES_DEFAULT:
            return response

        # Ensure authorization occurred.
        if not request_authorized(request):
            raise OsoException("authorize was not called during processing request.")

        return response

    return middleware

def RouteAuthorization(get_response):
    """Authorize route."""
    def middleware(request):
        authorize(request, resource=request)
        return get_response(request)

    return middleware
