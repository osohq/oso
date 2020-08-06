__version__ = '0.0.0'

import functools
from flask import g, current_app, _app_ctx_stack, request, Request
from werkzeug.exceptions import Forbidden

import oso
from oso import OsoException

class FlaskOso:
    """Flask specific functionality for oso."""
    def __init__(self, oso=None, app=None):
        self._app = app
        self._oso = None

        # TODO (dhatch): A few defaults for this dependending on what
        # other frameworks are in use.
        self._get_actor = lambda: g.current_user

        if self._app is not None:
            self.init_app(self._app)

        if oso is not None:
            self.set_oso(oso)

        # TODO config parameters

    ## Initialization

    def set_oso(self, oso):
        if oso == self._oso:
            return

        self._oso = oso
        self._oso.register_class(Request)

    def init_app(self, app):
        app.teardown_appcontext(self.teardown)
        app.before_request(self._provide_oso)

    def set_get_actor(self, func):
        """Provide a function that oso will use to get the current actor."""
        self._get_actor = func

    ## Middleware-like functions that affect every request.

    def require_authorization(self, app=None):
        """Enforce authorization on every request."""
        if app is None:
            app = self.app

        app.after_request(self._require_authorization)

    def perform_route_authorization(self, app=None):
        """Perform route authorization before every request.

        Route authorization will call :py:meth:`oso.Oso.is_allowed` with the
        current request (from ``flask.request``) as the resource and the method
        (from ``flask.request.method``) as the action.
        """
        if app is None:
            app = self.app

        app.before_request(self._perform_route_authorization)

    ## During request decorator or functions.

    def skip_authorization(self, reason=None):
        """opt-out of authorization for the current request."""
        _authorize_called()

    def authorize(self, resource, *, actor=None, action=None):
        if actor is None:
            actor = self.current_actor

        if action is None:
            action = request.method

        # TODO (dhatch): Broader resource mapping functionality?
        # Special handling for flask request as a resource.
        if resource == request:
            resource = request._get_current_object()

        allowed = self.oso.is_allowed(actor, action, resource)
        _authorize_called()

        if not allowed:
            raise Forbidden("Not authorized")

    @property
    def app(self):
        return self._app or current_app

    @property
    def oso(self):
        return self._oso

    @property
    def current_actor(self):
        return self._get_actor()

    ## Before / after
    def _provide_oso(self):
        if not hasattr(_app_ctx_stack.top, "oso_flask_oso"):
            _app_ctx_stack.top.oso_flask_oso = self

    def _perform_route_authorization(self):
        self.authorize(resource=request)

    def _require_authorization(self, response):
        if not request.url_rule:
            # No rule matched this request
            # Skip requiring authorization.
            # NOTE: (dhatch) Confirm this is a safe behavior, think through edge
            # cases.
            return response

        if not getattr(_app_ctx_stack.top, "oso_flask_authorize_called", False):
            raise OsoException("Authorize not called.")

        return response

    def teardown(self, exception):
        pass


# Decorators

def authorize(func=None, /, *, resource, actor=None, action=None):
    if func is not None:
        @functools.wraps(func)
        def wrap(*args, **kwargs):
            oso = _app_ctx_stack.top.oso_flask_oso

            oso.authorize(actor=actor, action=action, resource=resource)
            return func(*args, **kwargs)

        return wrap

    return functools.partial(authorize, actor=actor, action=action, resource=resource)

def skip_authorization(func=None, /, reason=None):
    if func is not None:
        @functools.wraps(func)
        def wrap(*args, **kwargs):
            oso = _app_ctx_stack.top.oso_flask_oso
            oso.skip_authorization(reason=reason)
            return func(*args, **kwargs)

        return wrap

    return functools.partial(skip_authorization, reason=reason)

def _authorize_called():
    """Mark current request as authorized."""
    _app_ctx_stack.top.oso_flask_authorize_called = True
