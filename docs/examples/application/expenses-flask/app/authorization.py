from dataclasses import dataclass
from flask import current_app, g, request, Blueprint
from oso import Oso, OsoException
from oso.extras import Http
from werkzeug.exceptions import BadRequest, Forbidden

bp = Blueprint("authorization", __name__)


@bp.before_app_request
def authorize_request():
    """Authorize the incoming request"""
    http = Http(path=request.path)
    if not current_app.oso.is_allowed(g.current_user, request.method, http):
        return Forbidden("Not Authorized!")


def authorize(action, resource):
    """Authorize whether the current user can perform `action` on `resource`"""
    if current_app.oso.is_allowed(g.current_user, action, resource):
        return resource
    else:
        raise Forbidden("Not Authorized!")


def init_oso(app):
    from .expense import Expense
    from .organization import Organization
    from .user import Actor, Guest, User

    oso = Oso()
    oso.register_class(Actor)
    oso.register_class(Guest)
    oso.register_class(User)
    oso.register_class(Expense)
    oso.register_class(Organization)

    for policy in app.config.get("OSO_POLICIES", []):
        oso.load_file(policy)

    app.oso = oso
