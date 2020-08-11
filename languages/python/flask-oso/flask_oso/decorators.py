import functools

from flask import g, current_app, request, Request

from .context import _app_oso


def authorize(func=None, resource=None, actor=None, action=None):
    """Flask route decorator.  Calls :py:meth:`FlaskOso.authorize` before the route.

    Parameters are the same as :py:meth:`FlaskOso.authorize`.

    .. warning::

        This decorator must come **after** the ``route`` decorator as shown below, otherwise authorization will not be
        checked.


    For example::

        @app.route("/")
        @authorize(resource=flask.request)
        def route():
            return "authorized"
    """
    if func is not None:

        @functools.wraps(func)
        def wrap(*args, **kwargs):
            oso = _app_oso()

            oso.authorize(actor=actor, action=action, resource=resource)
            return func(*args, **kwargs)

        return wrap

    return functools.partial(authorize, actor=actor, action=action, resource=resource)


def skip_authorization(func=None, reason=None):
    """Decorator to mark route as not requiring authorization.

    .. warning::

        This decorator must come after the ``route`` decorator.

    Causes use in conjunction with :py:meth:`FlaskOso.require_authorization` to
    silence errors on routes that do not need to be authorized.
    """
    if func is not None:

        @functools.wraps(func)
        def wrap(*args, **kwargs):
            oso = _app_oso()
            oso.skip_authorization(reason=reason)
            return func(*args, **kwargs)

        return wrap

    return functools.partial(skip_authorization, reason=reason)
