"""Tests the Polar API as an external consumer"""
import os
import pytest

from pathlib import Path

from polar.api import Http, Polar
from polar.exceptions import PolarRuntimeException, PolarApiException

from test_api_externals import Widget, DooDad, Actor, Company, get_frobbed, set_frobbed

try:
    # This import is required when running the rust version of the library
    # so that the fixture is registered with pytest.
    from polar.test_helpers import polar
except ImportError:
    pass

from polar.test_helpers import tell, qvar, query, oso_monkeypatch as polar_monkeypatch

# WOWHACK until API stabilizes.
if not os.getenv("OSO_COMPAT"):
    from polar.api import Predicate
else:
    from polar.api import Query

    class Predicate(Query):
        pass

    setattr(Polar, "register_class", Polar.register_python_class)
    setattr(Polar, "_to_python", Polar.to_python)
    setattr(Polar, "_to_polar_term", Polar.to_polar)
    setattr(Polar, "_load_queued_files", Polar._kb_load)
    setattr(Polar, "_query_pred", Polar.query)


# Set if running tests against old code
EXPECT_XFAIL_PASS = not bool(os.getenv("EXPECT_XFAIL_PASS", False))

## FIXTURES ##


@pytest.fixture
def load_policy(polar):
    # register all classes
    polar.register_class(Widget)
    polar.register_class(DooDad)
    polar.register_class(Actor)
    polar.register_class(Company)

    # import the test policy
    polar.load(Path(__file__).parent / "policies" / "test_api.polar")


default_company = Company(id="1", default_role="admin")


@pytest.fixture
def widget_in_company(polar_monkeypatch):
    return polar_monkeypatch.patch(Widget, "company", default_company)


@pytest.fixture
def actor_in_role(polar_monkeypatch):
    def _patch(role):
        return polar_monkeypatch.patch(Company, "role", role)

    return _patch


## TESTS ##


def test_register_class(polar, load_policy):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar._query_pred(
        Predicate(name="allow", args=(actor, action, resource))
    ).success


def test_allow(polar, load_policy):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, action, resource])
    ).success
    actor = Actor(name="president")
    assert polar._query_pred(
        Predicate(name="actorInRole", args=[actor, "admin", resource])
    ).success
    assert polar._query_pred(
        Predicate(name="allowRole", args=["admin", "create", resource])
    ).success


def test_method_resolution_order(polar, load_policy):
    set_frobbed([])
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, action, resource])
    ).success
    assert get_frobbed() == ["Widget"]
    set_frobbed([])
    resource = DooDad(id="2")
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, action, resource])
    ).success
    assert get_frobbed() == ["DooDad", "Widget"]


def test_cut(polar, load_policy):
    set_frobbed([])
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar._query_pred(
        Predicate(name="allow_with_cut", args=[actor, action, resource])
    ).success
    assert get_frobbed() == ["Widget"]
    set_frobbed([])
    resource = DooDad(id="2")
    assert polar._query_pred(
        Predicate(name="allow_with_cut", args=[actor, action, resource])
    ).success
    assert get_frobbed() == ["DooDad"]


def test_querystring_resource_map(polar, load_policy):
    assert polar._query_pred(
        Predicate(
            name="allow",
            args=[
                Actor(name="sam"),
                "what",
                Http(path="/widget/12", query={"param": "foo"}),
            ],
        )
    ).success
    assert not polar._query_pred(
        Predicate(
            name="allow", args=[Actor(name="sam"), "what", Http(path="/widget/12")]
        )
    ).success


def test_resource_mapping(polar, load_policy):
    # from flask import Flask, request, Response, g
    try:
        from flask import Flask, request, Response, g
    except ImportError:
        return pytest.skip("Flask not available in environment.")

    def set_user():
        g.user = Actor(name=request.headers["username"])

    app = Flask(__name__)
    app.before_request(set_user)

    @app.route("/widget/<int:id>")
    def get_widget(id):
        if not polar._query_pred(
            Predicate(
                name="allow",
                args=[g.user, request.method.lower(), Http(path=request.path)],
            )
        ).success:
            return Response("Denied", status=403)
        return Response("Ok", status=204)

    @app.route("/widget/", methods=["POST"])
    def create_widget():
        if not polar._query_pred(
            Predicate(
                name="allow",
                args=[g.user, request.method.lower(), Http(path=request.path)],
            )
        ).success:
            return Response("Denied", status=403)
        return Response("Ok", status=204)

    with app.test_client() as client:
        resp = client.get("/widget/1", headers=[("username", "guest")])
        assert resp.status_code == 204

        resp = client.post("/widget/", headers=[("username", "guest")])
        assert resp.status_code == 403

        resp = client.post("/widget/", headers=[("username", "president")])
        assert resp.status_code == 204


def test_patching(polar, widget_in_company, actor_in_role, load_policy):
    user = Actor("test")
    assert not polar._query_pred(
        Predicate(name="actorInRole", args=[user, "admin", Widget(id="1")])
    ).success
    with widget_in_company:
        with actor_in_role("admin"):
            assert polar._query_pred(
                Predicate(name="actorInRole", args=[user, "admin", Widget(id="1")])
            ).success
    assert not polar._query_pred(
        Predicate(name="actorInRole", args=[user, "admin", Widget(id="1")])
    ).success


## Instance Caching tests (move these somewhere else eventually)
def test_instance_round_trip(polar, query, qvar):
    # direct round trip
    user = Actor("sam")
    assert polar._to_python(polar._to_polar_term(user)) is user


@pytest.mark.xfail(
    EXPECT_XFAIL_PASS,
    reason="Instance literals are not instantiated for unify right now.",
)
def test_instance_initialization(polar, query, qvar):
    # test round trip through kb query
    user = Actor("sam")
    env = query('Actor{name:"sam"} = returned_user')[0]
    # Note this is not API compatible. It seems like
    # _query_str on the python version will return uninstantiated
    # external instances so another _to_python call is needed.
    # Might need a fix in test_helpers or somewhere esle.
    assert polar._to_python(env["returned_user"]) == user


def test_instance_from_external_call(polar, load_policy):
    user = Actor(name="guest")
    resource = Widget(id="1", name="name")
    assert polar._query_pred(
        Predicate(name="allow", args=[user, "frob", resource])
    ).success

    resource = Widget(id="2", name="name")
    assert not polar._query_pred(
        Predicate(name="allow", args=[user, "frob", resource])
    ).success


def test_load_input_checking(polar):
    with pytest.raises(PolarApiException):
        polar.load("unreal.py")
    with pytest.raises(PolarApiException):
        polar.load(Path(__file__).parent / "unreal.py")
    with pytest.raises(PolarApiException):
        polar.load(Path(__file__).parent / "unreal.polar")
    with pytest.raises(PolarApiException):
        polar.load(Path(__file__).parent / "unreal.pol")

    polar.load(Path(__file__).parent / "policies" / "test_api.polar")


@pytest.mark.xfail(
    EXPECT_XFAIL_PASS,
    reason="Lists are no longer converted to generators, but are returned as true lists.",
)
def test_return_list(polar, load_policy):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "invite"
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, action, resource])
    ).success


def test_type_fields(polar, load_policy):
    resource = Widget(id=1, name="goldfish")
    actor = Actor(name="elmo", id=1, widget=resource)
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, "keep", resource])
    ).success


def test_iter_fields(polar, load_policy):
    resource = Widget(id=1, name="stapler")
    actor = Actor(name="milton", id=1)
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, "can_have", resource])
    ).success


@pytest.mark.xfail(EXPECT_XFAIL_PASS, reason="Test relies on internal classes.")
def test_clear(polar, load_policy):
    old = Path(__file__).parent / "policies" / "load.pol"
    fails = Path(__file__).parent / "policies" / "reload_fail.pol"
    new = Path(__file__).parent / "policies" / "reload.pol"

    polar.clear()
    polar.load(old)

    actor = Actor(name="milton", id=1)
    resource = Widget(id=1, name="thingy")
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, "make", resource])
    ).success
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, "get", resource])
    ).success
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, "edit", resource])
    ).success
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, "delete", resource])
    ).success

    # raises exception because new policy file specifies on a class defined in the old file,
    # but not in the new file
    polar.clear()
    with pytest.raises(PolarRuntimeException):
        polar.load(fails)
        polar._load_queued_files()

    polar.clear()
    polar.load(new)
    assert polar._query_pred(
        Predicate(name="allow", args=[actor, "make", resource])
    ).success
    assert not polar._query_pred(
        Predicate(name="allow", args=[actor, "get", resource])
    ).success
    assert not polar._query_pred(
        Predicate(name="allow", args=[actor, "edit", resource])
    ).success
    assert not polar._query_pred(
        Predicate(name="allow", args=[actor, "delete", resource])
    ).success


if __name__ == "__main__":
    pytest.main([__file__])
