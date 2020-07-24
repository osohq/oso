from dataclasses import dataclass
from flask import current_app, g, request, Blueprint
from oso import Oso, OsoException
from werkzeug.exceptions import BadRequest, Forbidden

bp = Blueprint("authorization", __name__)


@bp.before_app_request
def authorize_request():
    """Authorize the incoming request"""
    try:
        if not current_app.oso.allow(g.current_user, request.method, request):
            return Forbidden("Not Authorized!")
    except OsoException as e:
        current_app.logger.exception(e)
        return BadRequest(e)


def authorize(action, resource):
    """Authorize whether the current user can perform `action` on `resource`"""
    if current_app.oso.allow(g.current_user, action, resource):
        return resource
    else:
        raise Forbidden("Not Authorized!")


def init_app(app):
    from .expense import Expense
    from .user import Actor, Guest, User

    oso = Oso()
    oso.register_class(Actor)
    oso.register_class(Guest)
    oso.register_class(User)
    oso.register_class(Expense)

    for policy in app.config.get("OSO_POLICIES", []):
        oso.load_file(policy)

    # force load to check for errors
    oso._load_queued_files()
    app.oso = oso
