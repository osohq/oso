"""Entrypoint to the expenses application"""

import flask
from flask import g, Flask
import os
from oso import Oso

from . import authorization, db, expense, organization, user


app = Flask(__name__)
oso = Oso()


def create_app(test_config=None):
    # create and configure the app
    app = Flask(__name__)
    app.config.from_mapping(
        DATABASE="expenses.db", OSO_POLICIES=["app/authorization.polar"]
    )

    # register DB handlers
    app.register_blueprint(db.bp)
    # regiester user handlers
    app.register_blueprint(user.bp)
    # register expenses routes
    app.register_blueprint(expense.bp)
    # register organizations routes
    app.register_blueprint(organization.bp)
    # register authorization handlers
    app.register_blueprint(authorization.bp)
    authorization.init_oso(app)

    #### Simple test route
    @app.route("/")
    def hello():
        return f"hello {g.current_user}"

    return app


def drop_into_pdb(app, exception):
    import sys
    import pdb
    import traceback

    traceback.print_exc()
    pdb.post_mortem(sys.exc_info()[2])


# somewhere in your code (probably if DEBUG is True)
flask.got_request_exception.connect(drop_into_pdb)

app = create_app()
