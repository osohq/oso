from flask import g, current_app, _app_ctx_stack, request, Request
from werkzeug.exceptions import Forbidden

from oso import OsoException, Oso

class FlaskOso:
    """oso flask plugin

    This plugin should be initialized with a flask app, either using the
    ``app`` parameter in the constructor, or by calling ``init_app`` after
    construction.

    The plugin most be initialized with a :py:class:`oso.Oso` instance before
    use, either by passing one to the constuctor or calling
    :py:meth:`FlaskOso.set_oso`.

    Authorization
    -------------
    ...

    Configuration
    -------------

    - :py:meth:`require_authorization`: Call to require at least one
      :py:meth:`FlaskOso.authorize` call for every request. If a call to
      :py:meth:`FlaskOso.authorize` is not made, an :py:class:`oso.OsoException`
      is raised.
    - :py:meth:`set_get_actor`
    """
    def __init__(self, oso=None, app=None):
        self._app = app
        self._oso = None

        def unauthorized():
            raise Forbidden("Unauthorized")
        self._unauthorized_action = unauthorized

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

    def set_unauthorized_action(self, func):
        """Set a function that will be called to handle an authorization failure.

        The default behavior is to raise a Forbidden exception, returning a 403
        response.
        """
        self._unauthorized_action = func

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
        # We use *is* here because == would actually need to get the request object.
        # We want to check that the resource is the proxy.
        if resource is request:
            resource = request._get_current_object()

        allowed = self.oso.is_allowed(actor, action, resource)
        _authorize_called()

        if not allowed:
            self._unauthorized_action()

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
        if not request.url_rule:
            # If request isn't going to match any route, just return and
            # let flask handle it the normal way.
            return

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

def _authorize_called():
    """Mark current request as authorized."""
    _app_ctx_stack.top.oso_flask_authorize_called = True
