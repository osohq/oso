from flask import current_app, g, Blueprint
import logging
import os
import sqlite3


bp = Blueprint("db", __name__)


@bp.before_app_request
def get_db():
    if "db" not in g:
        g.db = sqlite3.connect(
            current_app.config["DATABASE"], detect_types=sqlite3.PARSE_DECLTYPES
        )
        g.db.row_factory = sqlite3.Row


@bp.teardown_app_request
def close_db(e=None):
    db = g.pop("db", None)

    if db is not None:
        db.commit()
        db.close()


def query_db(query, args=(), one=False):
    cur = g.db.execute(query, args)
    rv = [
        dict((cur.description[idx][0], value) for idx, value in enumerate(row))
        for row in cur.fetchall()
    ]
    if rv:
        return rv[0] if one else rv
    if cur.lastrowid:
        return cur.lastrowid
    return None if one else []
