from dataclasses import dataclass
from flask import current_app, g, request, Blueprint
from werkzeug.exceptions import Unauthorized

from .db import query_db

bp = Blueprint("user", __name__)


class Actor:
    """base abstract user type"""

    pass


@dataclass
class User(Actor):
    """ "logged in" user - has an email address"""

    id: int
    email: str
    title: str
    location_id: int
    organization_id: int
    manager_id: int

    @classmethod
    def get(cls, id: int):
        record = query_db(
            "select id, email, title, location_id, organization_id, manager_id from users where email  = ?",
            [id],
            one=True,
        )
        if record:
            return cls(**record)
        else:
            raise Exception("user not found")

    @classmethod
    def lookup(cls, email: str):
        record = query_db(
            "select id, email, title, location_id, organization_id, manager_id from users where email  = ?",
            [email],
            one=True,
        )
        if record:
            return cls(**record)
        else:
            raise Exception("user not found")


class Guest(Actor):
    """unknown user"""

    def __str__(self):
        return "Guest"


@bp.before_app_request
def set_current_user():
    """Set the `current_user` from the authorization header (if present)"""
    if not "current_user" in g:
        email = request.headers.get("user")
        if email:
            try:
                g.current_user = User.lookup(email)
            except Exception as e:
                current_app.logger.exception(e)
                return Unauthorized("user not found")
        else:
            g.current_user = Guest()


@bp.route("/whoami")
def whoami():
    you = g.current_user
    if isinstance(you, User):
        organization = query_db(
            "select name from organizations where id = ?",
            [you.organization_id],
            one=True,
        )["name"]
        return f"You are {you.email}, the {you.title} at {organization}. (User ID: {you.id})"
