"""Middleware"""
from django.core.exceptions import PermissionDenied, ViewDoesNotExist, MiddlewareNotUsed
from django.conf import settings
from django.http import response
from django.shortcuts import redirect
from django.urls import resolve, reverse
from urllib.parse import urlencode

from django_oso.oso import reset_oso
from django_oso import Oso

from oso import OsoError

from .auth import request_authorized, authorize

# TODO (dhatch): Make this configurable.
# HTTP status codes that are permitted without authorization.
STATUS_CODES_WITHOUT_AUTHORIZATION = {
    401,
    403,
    404,
    405,
    500,
}


class OsoMiddleware:
    """Core oso middleware functionality

    Default behaviour is to check that authorization was applied
    at either the route or the view level.

    Authorization errors on route-level decisions return 404
    or reroute to the login page if the user is anonymous.

    View-level authorization errors return 404.

    Default inputs to the authorization decision for routes is the URL path
    and the view.

    Rules in oso policies can be written over requests using the ``HttpRequest``
    specializer:

    .. code-block:: polar

        allow(actor, action, resource: HttpRequest) if
            # Access request properties to perform authorization
            request.path = "/";
    """

    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        """Called before a request"""
        self.before_request(request)
        request = self.process_request(request)
        response = self.get_response(request)
        return self.process_response(request, response)

    def before_request(self, request):
        if settings.DEBUG:
            reset_oso()

    def get_current_user(self, request):
        """Get the current user. Defaults to ``request.user``"""
        return request.user

    def get_oso(self):
        return Oso

    def process_request(self, request):
        return request

    def process_response(self, request, response):
        if response.status_code == 403:
            return self.on_view_denied(request)
        elif response.status_code in STATUS_CODES_WITHOUT_AUTHORIZATION:
            pass
        else:
            if not request_authorized(request):
                raise OsoError("authorize was not called during processing request.")

        return response

    def process_view(self, request, view_func, view_args, view_kwargs):
        if resource := request.resolver_match:
            # print(request.resolver_match)
            actor = self.get_current_user(request)
            authorized = next(
                self.get_oso().query_rule("allow_route", actor, "view", resource),
                False,
            )
            request._oso_authorized = True

            if not authorized:
                return self.on_route_denied(request)

    def process_exception(self, request, exception):
        if isinstance(exception, (OsoError, PermissionError, PermissionDenied)):
            self.on_view_denied(request)

    def on_route_denied(self, request):
        """Handles authorization errors on route-level decisions"""
        if getattr(self.get_current_user(request), "is_anonymous", False):
            return redirect(
                reverse(settings.LOGIN_URL)
                + "?"
                + urlencode(dict(next=request.get_full_path()))
            )
        else:
            raise ViewDoesNotExist

    def on_view_denied(self, request):
        """Handles authorization errors on view-level decisions"""
        raise ViewDoesNotExist


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
        if response.status_code in STATUS_CODES_WITHOUT_AUTHORIZATION:
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
