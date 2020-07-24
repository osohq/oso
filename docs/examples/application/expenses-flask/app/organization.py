from dataclasses import dataclass
from flask import Blueprint, jsonify

from .authorization import authorize
from .db import query_db

bp = Blueprint("organization", __name__, url_prefix="/organizations")


@dataclass
class Organization:
    """Organization model"""

    name: int
    id: int = None

    @classmethod
    def lookup(cls, id: int):
        """Lookup an organization from the DB by id"""
        record = query_db("select id, name from organizations where id  = ?", [id], one=True,)
        return cls(**record)


@bp.route("/<int:id>", methods=["GET"])
def get_organization(id):
    return str(authorize("read", Organization.lookup(id)))
