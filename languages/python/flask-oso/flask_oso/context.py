"""Storage of oso information on app context."""
from flask import _app_ctx_stack

from oso import OsoError


def _app_context():
    """Get the app context. Use this instead of direct access to raise an appropriate error"""
    top = _app_ctx_stack.top
    if top is None:
        raise OsoError(
            "Application context doesn't exist. Did you use oso outside the context of a request? "
            "See https://flask.palletsprojects.com/en/1.1.x/appcontext/#manually-push-a-context"
        )

    return top


def _app_oso():
    """Get the flask oso plugin for the current app instance."""
    try:
        return _app_context().oso_flask_oso
    except AttributeError:
        raise OsoError(
            "No oso instance on current application. "
            "Did you forget to call init_app?"
        )
