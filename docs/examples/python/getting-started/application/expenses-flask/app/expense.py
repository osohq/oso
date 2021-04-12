from dataclasses import dataclass
from datetime import datetime
from flask import Blueprint, g, jsonify, request
from werkzeug.exceptions import BadRequest, NotFound

from .authorization import authorize
from .db import query_db
from .user import User

bp = Blueprint("expense", __name__, url_prefix="/expenses")


@dataclass
class Expense:
    """Expense model"""

    amount: int
    description: str
    user_id: int
    id: int = None

    def submitted_by(self):
        User.get(self.user_id)

    def save(self):
        now = datetime.now()
        id = query_db(
            """
            INSERT INTO expenses (amount, description, user_id, created_at, updated_at)
                VALUES(?, ?, ?, ?, ?) 
        """,
            [self.amount, self.description, self.user_id, now, now,],
        )
        self.id = id

    @classmethod
    def lookup(cls, id: int):
        """Lookup an expense from the DB by id"""
        record = query_db(
            "select id, amount, description, user_id from expenses where id  = ?",
            [id],
            one=True,
        )
        if record is None:
            raise NotFound()
        return cls(**record)


@bp.route("/<int:id>", methods=["GET"])
def get_expense(id):
    expense = Expense.lookup(id)
    return str(authorize("read", expense))


@bp.route("/submit", methods=["PUT"])
def submit_expense():
    expense_data = request.get_json(force=True)
    if not expense_data:
        raise BadRequest()
    # if no user id supplied, assume it is for the current user
    expense_data.setdefault("user_id", g.current_user.id)
    expense = Expense(**expense_data)
    expense.save()
    return str(expense)
