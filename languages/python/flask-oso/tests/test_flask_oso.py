"""Test the flask oso plugin."""
from pathlib import Path

import pytest
from flask import Flask
from oso import Oso, OsoError
from werkzeug.exceptions import Forbidden

from flask_oso import FlaskOso, authorize, skip_authorization


@pytest.fixture
def flask_app():
    return Flask("test")


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def user():
    return "user"


@pytest.fixture
def flask_oso(flask_app, oso, user):
    fo = FlaskOso(oso=oso, app=flask_app)
    fo.set_get_actor(lambda: user)
    return fo


@pytest.fixture
def simple_policy(oso):
    """Load a simple base policy into oso."""
    oso.load_file(Path(__file__).parent / "simple.polar")


@pytest.fixture
def app_ctx(flask_app):
    with flask_app.app_context():
        yield


def test_initialization_with_set(flask_app, oso, simple_policy, app_ctx, user):
    """Test that setting oso works correctly."""
    # Establish that an improperly initialized flask oso throws an exception.
    flask_oso = FlaskOso()
    flask_oso.set_get_actor(lambda: user)
    with pytest.raises(OsoError):
        flask_oso.authorize(action="read", resource="resource")

    # Works after set oso.
    flask_oso.set_oso(oso)
    flask_oso.authorize(action="read", resource="resource")


def test_initialization_with_init(flask_app, oso, simple_policy, app_ctx, user):
    # Works with oso init.
    flask_oso = FlaskOso(oso=oso)
    flask_oso.set_get_actor(lambda: user)
    flask_oso.authorize(action="read", resource="resource")


def test_authorize(flask_app, flask_oso, simple_policy, app_ctx):
    """Test that authorize function works correctly."""
    # Actor defaults to current actor.
    flask_oso.authorize("resource", action="read")

    # Overridden actor.
    with pytest.raises(Forbidden):
        flask_oso.authorize("resource", actor="other", action="read")

    flask_oso.authorize("other_resource", actor="other_user", action="read")

    # Request method action default
    with flask_app.test_request_context(method="GET"):
        flask_oso.authorize("action_resource")

    with flask_app.test_request_context(method="POST"):
        with pytest.raises(Forbidden):
            flask_oso.authorize("action_resource")

    flask_oso.set_get_actor(lambda: "other_user")
    flask_oso.authorize("other_resource", action="read")


def test_require_authorization(flask_app, flask_oso, app_ctx, simple_policy):
    flask_oso.require_authorization(flask_app)
    flask_app.testing = True

    @flask_app.route("/")
    def hello():
        return "Hello"

    # Don't call authorize.
    with pytest.raises(OsoError):
        with flask_app.test_client() as c:
            c.get("/")

    @flask_app.route("/auth")
    def auth():
        flask_oso.authorize("resource", action="read")
        return "Hello"

    with flask_app.test_client() as c:
        resp = c.get("/auth")
        assert resp.status_code == 200

    # Decorator works
    @flask_app.route("/decorator")
    @authorize(action="read", resource="resource")
    def decorated():
        return "Hello"

    with flask_app.test_client() as c:
        resp = c.get("/decorator")
        assert resp.status_code == 200

    # Skip auth silences error
    @flask_app.route("/open")
    @skip_authorization
    def open():
        return "open"

    with flask_app.test_client() as c:
        resp = c.get("/open")
        assert resp.status_code == 200

    # 404 doesn't require authorization
    with flask_app.test_client() as c:
        resp = c.get("/nonexistent")
        assert resp.status_code == 404

    # Server error does
    @flask_app.route("/500")
    def server_error():
        raise Exception("You messed this one up")

    flask_app.testing = False
    # Ensure that requiring authorization doesn't interfere with surfacing
    # other exceptions that occur during the request.
    with flask_app.test_client() as c:
        resp = c.get("/500")
        assert resp.status_code == 500


def test_route_authorization(flask_oso, oso, flask_app, app_ctx):
    """Test that route authorization middleware works."""
    flask_oso.perform_route_authorization(app=flask_app)
    flask_app.testing = True

    @flask_app.route("/test_route", methods=("GET",))
    def test():
        return "Test"

    with flask_app.test_client() as c:
        with pytest.raises(OsoError) as e:
            c.get("/test_route")
        assert "Query for undefined rule `allow`" in str(e)

    # Add rule to policy.
    oso.load_str('allow("user", "GET", _: Request{path: "/test_route"});')

    flask_oso.set_get_actor(lambda: "other_user")
    with flask_app.test_client() as c:
        assert c.get("/test_route").status_code == 403

    flask_oso.set_get_actor(lambda: "user")
    with flask_app.test_client() as c:
        assert c.get("/test_route").status_code == 200

    # Confirm that route authorization doesn't mess with errors.
    with flask_app.test_client() as c:
        assert c.get("/not_a_route").status_code == 404

    with flask_app.test_client() as c:
        assert c.post("/test_route").status_code == 405


def test_route_authorizaton_manual(flask_oso, oso, flask_app, app_ctx):
    """Perform route auth manually."""
    flask_app.testing = True

    from flask import request

    @flask_app.route("/test_route")
    @authorize(resource=request)
    def auth():
        return "authed"

    with flask_app.test_client() as c:
        with pytest.raises(OsoError) as e:
            c.get("/test_route")
        assert "Query for undefined rule `allow`" in str(e)

    # Add rule
    oso.load_str('allow("user", "GET", _: Request{path: "/test_route"});')

    flask_oso.set_get_actor(lambda: "other_user")
    with flask_app.test_client() as c:
        assert c.get("/test_route").status_code == 403

    flask_oso.set_get_actor(lambda: "user")
    with flask_app.test_client() as c:
        assert c.get("/test_route").status_code == 200


def test_custom_unauthorize(flask_oso, oso, flask_app, app_ctx):
    """Test that a custom unauthorize handler can be provided."""
    auth_failed = False

    def unauth():
        nonlocal auth_failed
        auth_failed = True

    flask_oso.set_unauthorized_action(unauth)

    # Add rule
    oso.load_str('allow(_, "not bad", _);')

    flask_oso.authorize(resource="fail!", action="bad")
    assert auth_failed


def test_no_oso_error(flask_app, oso):
    """Test that using authorize without init app throws an error."""
    with pytest.raises(OsoError, match="Application context"):

        @authorize(resource="test")
        def orm_function():
            return "model"

        orm_function()

    with flask_app.app_context():
        with pytest.raises(OsoError, match="init_app"):

            @flask_app.route("/")
            @authorize(resource="test")
            def route():
                return "test"

            flask_app.testing = True
            with flask_app.test_client() as c:
                c.get("/").status_code
