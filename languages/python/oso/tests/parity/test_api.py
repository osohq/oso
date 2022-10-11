"""Tests the Polar API as an external consumer"""
import os
from contextlib import contextmanager
from pathlib import Path

import pytest

from polar import Predicate
from polar.exceptions import (
    InvalidIteratorError,
    PolarFileExtensionError,
    PolarFileNotFoundError,
    PolarRuntimeError,
)

from .test_api_externals import (
    Company,
    DooDad,
    Http,
    PathMapper,
    User,
    Widget,
    get_frobbed,
    set_frobbed,
)

# Set if running tests against old code
EXPECT_XFAIL_PASS = not bool(os.getenv("EXPECT_XFAIL_PASS", False))

# *** FIXTURES *** #


@pytest.fixture()
def register_classes(polar):
    polar.register_class(Company)
    polar.register_class(Widget)
    polar.register_class(DooDad)
    polar.register_class(Http)
    polar.register_class(PathMapper)


@pytest.fixture
def load_policy(polar):
    polar.register_class(User)
    polar.load_file(Path(__file__).parent / "policies" / "test_api.polar")


default_company = Company(id="1", default_role="admin")


@pytest.fixture
def widget_in_company(monkeypatch):
    @contextmanager
    def patch():
        with monkeypatch.context() as m:
            m.setattr(Widget, "company", lambda *args: default_company)
            yield

    return patch


@pytest.fixture
def actor_in_role(monkeypatch):
    @contextmanager
    def patch(role):
        with monkeypatch.context() as m:
            m.setattr(Company, "role", lambda *args: role)
            yield

    return patch


# *** TESTS *** #


def test_register_class(polar, register_classes, load_policy, query):
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert query(Predicate(name="allow", args=(actor, action, resource)))


def test_is_allowed(polar, register_classes, load_policy, query):
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert query(Predicate(name="allow", args=[actor, action, resource]))
    actor = User(name="president")
    assert query(Predicate(name="actorInRole", args=[actor, "admin", resource]))
    assert query(Predicate(name="allowRole", args=["admin", "create", resource]))


def test_method_resolution_order(polar, register_classes, load_policy, query):
    set_frobbed([])
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert query(Predicate(name="allow", args=[actor, action, resource]))
    assert get_frobbed() == ["Widget"]

    # DooDad is a Widget
    set_frobbed([])
    resource = DooDad(id="2")
    assert query(Predicate(name="allow", args=[actor, action, resource]))
    assert get_frobbed() == ["DooDad", "Widget"]


def test_cut(polar, register_classes, load_policy, query):
    set_frobbed([])
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert query(Predicate(name="allow_with_cut", args=[actor, action, resource]))
    assert get_frobbed() == ["Widget"]
    set_frobbed([])
    resource = DooDad(id="2")
    assert query(Predicate(name="allow_with_cut", args=[actor, action, resource]))
    assert get_frobbed() == ["DooDad"]


def test_querystring_resource_map(polar, register_classes, load_policy, query):
    assert query(
        Predicate(
            name="allow",
            args=[
                User(name="sam"),
                "what",
                Http(path="/widget/12", query={"param": "foo"}),
            ],
        )
    )
    assert not query(
        Predicate(
            name="allow", args=[User(name="sam"), "what", Http(path="/widget/12")]
        )
    )


def test_resource_mapping(polar, register_classes, load_policy, query):
    # from flask import Flask, request, Response, g
    try:
        from flask import Flask, Response, g, request
    except ImportError:
        return pytest.skip("Flask not available in environment.")

    def set_user():
        g.user = User(name=request.headers["username"])

    app = Flask(__name__)
    app.before_request(set_user)

    @app.route("/widget/<int:id>")
    def get_widget(id):
        if not query(
            Predicate(
                name="allow",
                args=[g.user, request.method.lower(), Http(path=request.path)],
            )
        ):
            return Response("Denied", status=403)
        return Response("Ok", status=204)

    @app.route("/widget/", methods=["POST"])
    def create_widget():
        if not query(
            Predicate(
                name="allow",
                args=[g.user, request.method.lower(), Http(path=request.path)],
            )
        ):
            return Response("Denied", status=403)
        return Response("Ok", status=204)

    with app.test_client() as client:
        resp = client.get("/widget/1", headers=[("username", "guest")])
        assert resp.status_code == 204

        resp = client.post("/widget/", headers=[("username", "guest")])
        assert resp.status_code == 403

        resp = client.post("/widget/", headers=[("username", "president")])
        assert resp.status_code == 204


def test_patching(
    polar, widget_in_company, actor_in_role, register_classes, load_policy, query
):
    user = User("test")
    assert not query(
        Predicate(name="actorInRole", args=[user, "admin", Widget(id="1")])
    )
    with widget_in_company():
        with actor_in_role("admin"):
            assert query(
                Predicate(name="actorInRole", args=[user, "admin", Widget(id="1")])
            )
    assert not query(
        Predicate(name="actorInRole", args=[user, "admin", Widget(id="1")])
    )


# Instance Caching tests (move these somewhere else eventually)
def test_instance_round_trip(polar, query, qvar):
    # direct round trip
    user = User("sam")
    assert polar.host.to_python(polar.host.to_polar(user)) is user


@pytest.mark.xfail(
    EXPECT_XFAIL_PASS,
    reason="Instance literals are not instantiated for unify right now.",
)
def test_instance_initialization(polar, query, qvar):
    # test round trip through kb query
    user = User("sam")
    env = query('new User(name:"sam") = returned_user')[0]
    assert polar.host.to_python(env["returned_user"]) == user

    env = query('new User(name:"sam") = returned_user')[0]
    assert polar.host.to_python(env["returned_user"]) == user


def test_instance_from_external_call(polar, register_classes, load_policy, query):
    user = User(name="guest")
    resource = Widget(id="1", name="name")
    assert query(Predicate(name="allow", args=[user, "frob", resource]))

    resource = Widget(id="2", name="name")
    assert not query(Predicate(name="allow", args=[user, "frob", resource]))


def test_load_input_checking(polar, register_classes, query):
    with pytest.raises(PolarFileExtensionError):
        polar.load_file("unreal.py")
    with pytest.raises(PolarFileExtensionError):
        polar.load_file(Path(__file__).parent / "unreal.py")
    with pytest.raises(PolarFileNotFoundError):
        polar.load_file(Path(__file__).parent / "unreal.polar")
    with pytest.raises(PolarFileExtensionError):
        polar.load_file(Path(__file__).parent / "unreal.pol")

    polar.load_file(Path(__file__).parent / "policies" / "test_api.polar")


@pytest.mark.xfail(
    EXPECT_XFAIL_PASS,
    reason="Lists are no longer converted to generators, but are returned as true lists.",
)
def test_return_list(polar, load_policy, query):
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "invite"
    assert query(Predicate(name="allow", args=[actor, action, resource]))


def test_type_fields(polar, register_classes, load_policy, query):
    resource = Widget(id=1, name="goldfish")
    actor = User(name="elmo", id=1, widget=resource)
    assert query(Predicate(name="allow", args=[actor, "keep", resource]))


def test_iter_fields(polar, register_classes, load_policy, query):
    resource = Widget(id=1, name="stapler")
    actor = User(name="milton", id=1)
    assert query(Predicate(name="allow", args=[actor, "can_have", resource]))
    with pytest.raises(InvalidIteratorError):
        query(Predicate(name="allow", args=[actor, "tries_to_get", resource]))


@pytest.mark.xfail(EXPECT_XFAIL_PASS, reason="Test relies on internal classes.")
def test_clear_rules(polar, load_policy, query):
    old = Path(__file__).parent / "policies" / "load.pol"
    fails = Path(__file__).parent / "policies" / "reload_fail.pol"
    new = Path(__file__).parent / "policies" / "reload.pol"

    polar.clear_rules()
    polar.load_file(old)

    actor = User(name="milton", id=1)
    resource = Widget(id=1, name="thingy")
    assert query(Predicate(name="allow", args=[actor, "make", resource]))
    assert query(Predicate(name="allow", args=[actor, "get", resource]))
    assert query(Predicate(name="allow", args=[actor, "edit", resource]))
    assert query(Predicate(name="allow", args=[actor, "delete", resource]))

    # raises exception because new policy file specifies on a class defined in the old file,
    # but not in the new file
    polar.clear_rules()
    with pytest.raises(PolarRuntimeError):
        polar.load_file(fails)

    polar.clear_rules()
    polar.load_file(new)
    assert query(Predicate(name="allow", args=[actor, "make", resource]))
    assert not query(Predicate(name="allow", args=[actor, "get", resource]))
    assert not query(Predicate(name="allow", args=[actor, "edit", resource]))
    assert not query(Predicate(name="allow", args=[actor, "delete", resource]))


if __name__ == "__main__":
    pytest.main([__file__])
