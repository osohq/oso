from flask import g, current_app, request, Request
from werkzeug.exceptions import Forbidden

from oso import OsoError, Oso

from .context import _app_context


class FlaskOso:
    """oso flask plugin

    This plugin must be initialized with a flask app, either using the
    ``app`` parameter in the constructor, or by calling :py:meth:`init_app` after
    construction.

    The plugin must be initialized with an :py:class:`oso.Oso` instance before
    use, either by passing one to the constructor or calling
    :py:meth:`set_oso`.

    **Authorization**

    - :py:meth:`FlaskOso.authorize`: Check whether an actor, action and resource is
      authorized. Integrates with flask to provide defaults for actor & action.

    **Configuration**

    - :py:meth:`require_authorization`: Require at least one
      :py:meth:`FlaskOso.authorize` call for every request.
    - :py:meth:`set_get_actor`: Override how oso determines the actor
      associated with a request if none is provided to :py:meth:`FlaskOso.authorize`.
    - :py:meth:`set_unauthorized_action`: Control how :py:meth:`FlaskOso.authorize`
      handles an unauthorized request.
    - :py:meth:`perform_route_authorization`:
      Call `authorize(resource=flask.request)` before every request.
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
        """Set the oso instance to use for authorization

        Must be called if ``oso`` is not provided to the constructor.
        """
        if oso == self._oso:
            return

        self._oso = oso
        self._oso.register_class(Request)

    def init_app(self, app):
        """Initialize ``app`` for use with this instance of ``FlaskOso``.

        Must be called if ``app`` isn't provided to the constructor.
        """
        app.teardown_appcontext(self.teardown)
        app.before_request(self._provide_oso)

    def set_get_actor(self, func):
        """Provide a function that oso will use to get the current actor.

        :param func: A function to call with no parameters to get the actor if
                     it is not provided to :py:meth:`FlaskOso.authorize`. The return value
                     is used as the actor.
        """
        self._get_actor = func

    def set_unauthorized_action(self, func):
        """Set a function that will be called to handle an authorization failure.

        The default behavior is to raise a Forbidden exception, returning a 403
        response.

        :param func: A function to call with no parameters when a request is
                     not authorized.
        """
        self._unauthorized_action = func

    ## Middleware-like functions that affect every request.

    def require_authorization(self, app=None):
        """Enforce authorization on every request to ``app``.

        :param app: The app to require authorization for. Can be omitted if
                    the ``app`` parameter was used in the ``FlaskOso``
                    constructor.

        If :py:meth:`FlaskOso.authorize` is not called during the request processing,
        raises an :py:class:`oso.OsoError`.

        Call :py:meth:`FlaskOso.skip_authorization` to skip this check for a particular
        request.
        """
        if app is None:
            app = self.app

        app.after_request(self._require_authorization)

    def perform_route_authorization(self, app=None):
        """Perform route authorization before every request.

        Route authorization will call :py:meth:`oso.Oso.is_allowed` with the
        current request (from ``flask.request``) as the resource and the method
        (from ``flask.request.method``) as the action.

        :param app: The app to require authorization for. Can be omitted if
                    the ``app`` parameter was used in the ``FlaskOso``
                    constructor.
        """
        if app is None:
            app = self.app

        app.before_request(self._perform_route_authorization)

    ## During request decorator or functions.

    def skip_authorization(self, reason=None):
        """Opt-out of authorization for the current request.

        Will prevent ``require_authorization`` from causing an error.

        See also: :py:func:`flask_oso.skip_authorization` for a route decorator version.
        """
        _authorize_called()

    def authorize(self, resource, *, actor=None, action=None):
        """Check whether the current request should be allowed.

        Calls :py:meth:`oso.Oso.is_allowed` to check authorization. If a request
        is unauthorized, raises a ``werkzeug.exceptions.Forbidden``
        exception.  This behavior can be controlled with
        :py:meth:`set_unauthorized_action`.

        :param actor: The actor to authorize. Defaults to ``flask.g.current_user``.
                      Use :py:meth:`set_get_actor` to override.
        :param action: The action to authorize. Defaults to
                       ``flask.request.method``.
        :param resource: The resource to authorize.  The flask request object
                         (``flask.request``) can be passed to authorize a
                         request based on route path or other request properties.

        See also: :py:func:`flask_oso.authorize` for a route decorator version.
        """
        if actor is None:
            try:
                actor = self.current_actor
            except AttributeError as e:
                raise OsoError(
                    "Getting the current actor failed. "
                    "You may need to override the current actor function with "
                    "FlaskOso#set_get_actor"
                ) from e

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
        top = _app_context()
        if not hasattr(top, "oso_flask_oso"):
            top.oso_flask_oso = self

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

        if not getattr(_app_context(), "oso_flask_authorize_called", False):
            raise OsoError("Authorize not called.")

        return response

    def teardown(self, exception):
        pass


def _authorize_called():
    """Mark current request as authorized."""
    _app_context().oso_flask_authorize_called = True
