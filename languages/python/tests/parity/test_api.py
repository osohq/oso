"""Tests the Polar API as an external consumer"""
import os
import pytest

from pathlib import Path

from polar.api import Http, Polar, Query
from polar.exceptions import PolarRuntimeException, PolarApiException

from test_api_externals import Widget, DooDad, Actor, Company, get_frobbed, set_frobbed

try:
    # This import is required when running the rust version of the library
    # so that the fixture is registered with pytest.
    from polar.test_helpers import polar
except ImportError:
    pass

from polar.test_helpers import tell, qvar, query, oso_monkeypatch as polar_monkeypatch

# Set if running tests against old code
EXPECT_XFAIL_PASS = not bool(os.getenv("EXPECT_XFAIL_PASS", False))

## FIXTURES ##


@pytest.fixture
def load_policy(polar):
    # register all classes
    polar.register_python_class(Widget)
    polar.register_python_class(DooDad)
    polar.register_python_class(Actor)
    polar.register_python_class(Company)

    # import the test policy
    if EXPECT_XFAIL_PASS:
        pytest.xfail(reason="Doesn't parse.")

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


def test_register_python_class(polar, load_policy):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar.query(Query(name="allow", args=(actor, action, resource))).success


def test_allow(polar, load_policy):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar.query(Query(name="allow", args=[actor, action, resource])).success
    actor = Actor(name="president")
    assert polar.query(
        Query(name="actorInRole", args=[actor, "admin", resource])
    ).success
    assert polar.query(
        Query(name="allowRole", args=["admin", "create", resource])
    ).success


def test_method_resolution_order(polar, load_policy):
    set_frobbed([])
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar.query(Query(name="allow", args=[actor, action, resource])).success
    assert get_frobbed() == ["Widget"]
    set_frobbed([])
    resource = DooDad(id="2")
    assert polar.query(Query(name="allow", args=[actor, action, resource])).success
    assert get_frobbed() == ["DooDad", "Widget"]


def test_cut(polar, load_policy):
    set_frobbed([])
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar.query(
        Query(name="allow_with_cut", args=[actor, action, resource])
    ).success
    assert get_frobbed() == ["Widget"]
    set_frobbed([])
    resource = DooDad(id="2")
    assert polar.query(
        Query(name="allow_with_cut", args=[actor, action, resource])
    ).success
    assert get_frobbed() == ["DooDad"]


def test_querystring_resource_map(polar, load_policy):
    assert polar.query(
        Query(
            name="allow",
            args=[
                Actor(name="sam"),
                "what",
                Http(path="/widget/12", query={"param": "foo"}),
            ],
        )
    ).success
    assert not polar.query(
        Query(name="allow", args=[Actor(name="sam"), "what", Http(path="/widget/12")])
    ).success


def test_resource_mapping(polar, load_policy):
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
        if not polar.query(
            Query(
                name="allow",
                args=[g.user, request.method.lower(), Http(path=request.path)],
            )
        ).success:
            return Response("Denied", status=403)
        return Response("Ok", status=204)

    @app.route("/widget/", methods=["POST"])
    def create_widget():
        if not polar.query(
            Query(
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


@pytest.mark.xfail(EXPECT_XFAIL_PASS, reason="Monkey patch not implemented.")
def test_patching(polar, widget_in_company, actor_in_role, load_policy):
    user = Actor("test")
    assert not polar.query(
        Query(name="actorInRole", args=[user, "admin", Widget(id="1")])
    ).success
    with widget_in_company:
        with actor_in_role("admin"):
            assert polar.query(
                Query(name="actorInRole", args=[user, "admin", Widget(id="1")])
            ).success
    assert not polar.query(
        Query(name="actorInRole", args=[user, "admin", Widget(id="1")])
    ).success


## Instance Caching tests (move these somewhere else eventually)
@pytest.mark.xfail(
    EXPECT_XFAIL_PASS, reason="Polar object has no attribute 'to_polar'."
)
def test_instance_round_trip(polar, query, qvar):
    # direct round trip
    user = Actor("sam")
    assert polar.to_python(polar.to_polar(user)) is user

    # test round trip through kb query
    env = query('Actor{name:"sam"} = returned_user')[0]
    assert polar.to_python(env["returned_user"]).__dict__ == user.__dict__

    # test instance round trip through api query
    returned_user = polar.to_python(
        qvar(Query(name="=", args=[user, "returned_user"]), "returned_user")[0]
    )
    assert returned_user.__dict__ is user.__dict__


def test_instance_from_external_call(polar, load_policy):
    user = Actor(name="guest")
    resource = Widget(id="1", name="name")
    assert polar.query(Query(name="allow", args=[user, "frob", resource])).success

    resource = Widget(id="2", name="name")
    assert not polar.query(Query(name="allow", args=[user, "frob", resource])).success


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


@pytest.mark.xfail(EXPECT_XFAIL_PASS, reason="There is no KB to check")
def test_default_load_policy():
    polar = Polar()
    polar.load(Path(__file__).parent / "policies" / "test_api.polar")
    # Policy is lazily added; not facts yet added
    assert len(polar._kb.facts) == 0

    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert polar.query(Query(name="allow", args=[actor, action, resource])).success

    # Confirm that facts are now loaded
    kb_len = len(polar._kb.facts)
    assert kb_len > 0

    # Confirm that duplicate policy files are not added
    polar.load(Path(__file__).parent / "policies" / "test_api.polar")
    polar.load(Path(__file__).parent / "policies" / "test_api.polar")
    assert len(polar._kb.facts) == kb_len


def test_return_list(polar, load_policy):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "invite"
    assert polar.query(Query(name="allow", args=[actor, action, resource])).success


def test_type_fields(polar, load_policy):
    resource = Widget(id=1, name="goldfish")
    actor = Actor(name="elmo", id=1, widget=resource)
    assert polar.query(Query(name="allow", args=[actor, "keep", resource])).success


def test_iter_fields(polar, load_policy):
    resource = Widget(id=1, name="stapler")
    actor = Actor(name="milton", id=1)
    assert polar.query(Query(name="allow", args=[actor, "can_have", resource])).success


def test_clear(polar, load_policy):
    old = Path(__file__).parent / "policies" / "load.pol"
    fails = Path(__file__).parent / "policies" / "reload_fail.pol"
    new = Path(__file__).parent / "policies" / "reload.pol"

    polar.clear()
    polar.load(old)

    actor = Actor(name="milton", id=1)
    resource = Widget(id=1, name="thingy")
    assert polar.query(Query(name="allow", args=[actor, "make", resource])).success
    assert polar.query(Query(name="allow", args=[actor, "get", resource])).success
    assert polar.query(Query(name="allow", args=[actor, "edit", resource])).success
    assert polar.query(Query(name="allow", args=[actor, "delete", resource])).success

    # raises exception because new policy file specifies on a class defined in the old file,
    # but not in the new file
    polar.clear()
    with pytest.raises(PolarRuntimeException):
        polar.load(fails)
        polar._kb_load()

    polar.clear()
    polar.load(new)
    assert polar.query(Query(name="allow", args=[actor, "make", resource])).success
    assert not polar.query(Query(name="allow", args=[actor, "get", resource])).success
    assert not polar.query(Query(name="allow", args=[actor, "edit", resource])).success
    assert not polar.query(
        Query(name="allow", args=[actor, "delete", resource])
    ).success


@pytest.mark.skip(
    reason="Used to determine behavior, unnecessary for now because we aren't exposing `isa()`"
)
def test_isa_api(polar, tell):
    polar.clear()
    polar_tell("isa(x, y) := isa(x, y);")
    polar_tell("isa_widget(w, id, name) := isa(w, Widget{id: id, name: name});")
    polar_tell("widget_isa(w, id, name) := isa(Widget{id: id, name: name}, x);")

    doodad = DooDad(id="1", name="bob")
    widget = Widget(id="2", name="joe")
    widget2 = Widget(id="3", name="james")

    assert polar.query(Query(name="isa", args=[doodad, widget])).success
    assert not polar.query(
        Query(name="isa_widget", args=[widget, "3", "james"])
    ).success
    assert polar.query(Query(name="widget_isa", args=[widget])).success
    assert polar.query(Query(name="isa_widget", args=[widget, "2", "joe"])).success

    # this shouldn't succeed because they have different fields, but it does
    assert polar.query(Query(name="isa", args=[widget, widget2])).success


if __name__ == "__main__":
    pytest.main([__file__])
