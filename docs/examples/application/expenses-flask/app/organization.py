from dataclasses import dataclass
from flask import Blueprint, jsonify

from .db import query_db

bp = Blueprint("expenses", __name__, url_prefix="/expenses")


@dataclass
class Expense:
    """Expense model"""

    id: int
    amount: int
    description: str
    user_id: int

    @classmethod
    def lookup(cls, id: int):
        """Lookup an expense from the DB by id"""
        record = query_db(
            "select id, amount, description, user_id from expenses where id  = ?",
            [id],
            one=True,
        )
        return cls(**record)


@bp.route("/<int:id>", methods=["GET"])
def get_expense(id):
    return str(Expense.lookup(id))


@bp.route("/", methods=["GET"])
def list_expenses():
    expenses = [
        str(Expense(**record))
        for record in query_db("SELECT amount, description, user_id FROM expenses")
    ]
    return jsonify(expenses)
