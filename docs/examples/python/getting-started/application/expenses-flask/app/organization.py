from dataclasses import dataclass
from flask import Blueprint, jsonify
from werkzeug.exceptions import NotFound

from .authorization import authorize
from .db import query_db

bp = Blueprint("organization", __name__, url_prefix="/organizations")


@dataclass
class Organization:
    """Organization model"""

    name: str
    id: int = None

    @classmethod
    def lookup(cls, id: int):
        """Lookup an organization from the DB by id"""
        record = query_db(
            "select id, name from organizations where id  = ?", [id], one=True,
        )
        if record is None:
            raise NotFound()
        return cls(**record)


@bp.route("/<int:id>", methods=["GET"])
def get_organization(id):
    organization = Organization.lookup(id)
    return str(authorize("read", organization))
