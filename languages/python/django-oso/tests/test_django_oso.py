from pathlib import Path
import sys

import pytest

from django.conf import settings
from django.test import RequestFactory
from django.core.exceptions import PermissionDenied

from django_oso.oso import Oso, reset_oso
from django_oso.auth import authorize, authorize_model
from polar.errors import UnsupportedError

from oso import OsoError


@pytest.fixture(autouse=True)
def reset():
    reset_oso()


@pytest.fixture
def simple_policy():
    """Load simple authorization policy."""
    Oso.load_file(Path(__file__).parent / "simple.polar")


@pytest.fixture
def partial_policy():
    """Load partial authorization policy."""
    Oso.load_file(Path(__file__).parent / "partial.polar")


def test_policy_autoload():
    """Test that policies are loaded from policy directory."""
    # These rules are added by the policies in the test app.
    assert next(Oso.query_rule("policy_load_test", 1))
    assert next(Oso.query_rule("policy_load_test", 2))


def test_model_registration():
    """Test that models are automatically registered with the policy."""
    from test_app import models
    from oso import Variable

    assert (
        next(Oso.query_rule("models", models.TestRegistration(), Variable("x")))[
            "bindings"
        ]["x"]
        == 1
    )
    assert (
        next(Oso.query_rule("models", models.TestRegistration2(), Variable("x")))[
            "bindings"
        ]["x"]
        == 2
    )


def test_authorize(rf, simple_policy):
    """Test that authorize function works."""
    request = rf.get("/")

    # No defaults
    authorize(request, actor="user", action="read", resource="resource")

    # Default action
    authorize(request, actor="user", resource="action_resource")

    # Default actor
    request.user = "user"
    authorize(request, resource="action_resource")

    # Not authorized
    with pytest.raises(PermissionDenied):
        authorize(request, "resource", actor="other", action="read")


def test_require_authorization(client, settings, simple_policy):
    """Test that require authorization middleware works."""
    settings.MIDDLEWARE.append("django_oso.middleware.RequireAuthorization")

    with pytest.raises(OsoError):
        response = client.get("/")

    response = client.get("/auth/")
    assert response.status_code == 200

    response = client.get("/auth_decorated_fail/")
    assert response.status_code == 403

    response = client.get("/auth_decorated/")
    assert response.status_code == 200

    # 404 gets through
    response = client.get("/notfound/")
    assert response.status_code == 404

    # 500 gets through
    response = client.get("/error/")
    assert response.status_code == 500


def test_route_authorization(client, settings, simple_policy):
    """Test route authorization middleware"""
    settings.MIDDLEWARE.append("django.contrib.sessions.middleware.SessionMiddleware")
    settings.MIDDLEWARE.append(
        "django.contrib.auth.middleware.AuthenticationMiddleware"
    )
    settings.MIDDLEWARE.append("django_oso.middleware.RouteAuthorization")

    response = client.get("/a/")
    assert response.status_code == 403

    response = client.get("/b/")
    assert response.status_code == 403

    Oso.load_str('allow(_, "GET", _: HttpRequest{path: "/a/"});')
    response = client.get("/a/")
    assert response.status_code == 200

    response = client.post("/a/")
    assert response.status_code == 403

    # Django runs url resolving after middleware, so there is no way
    # for the route authorization middleware to determine if the response
    # would be a 404 and not apply authorization.
    response = client.get("/notfound/")
    assert response.status_code == 403


@pytest.mark.django_db
def test_partial(rf, settings, partial_policy):
    from test_app.models import Post

    posts = [
        Post(name="test", is_private=False, timestamp=1).save(),
        Post(name="test_past", is_private=False, timestamp=-1).save(),
        Post(name="test_public", is_private=False, timestamp=1).save(),
        Post(name="test_private", is_private=True, timestamp=1).save(),
        Post(name="test_private_2", is_private=True, timestamp=1).save(),
        Post(name="test_option", is_private=False, timestamp=1, option=True).save(),
    ]

    request = rf.get("/")
    request.user = "test_user"

    authorize_filter = authorize_model(request, action="get", model="test_app::Post")
    assert (
        str(authorize_filter)
        == "(AND: ('is_private', False), ('timestamp__gt', 0), ('option', None))"
    )

    q = Post.objects.filter(authorize_filter)
    assert q.count() == 2

    request = rf.get("/")
    request.user = "test_admin"

    authorize_filter = authorize_model(request, action="get", model=Post)
    assert str(authorize_filter) == "(AND: )"

    q = Post.objects.filter(authorize_filter)
    assert q.count() == len(posts)

    q = Post.objects.authorize(request, action="get")
    assert q.count() == len(posts)


@pytest.mark.django_db
def test_partial_errors(rf, settings):
    from test_app.models import Post

    Post(name="test", is_private=False, timestamp=1).save()
    Post(name="test_past", is_private=False, timestamp=-1).save()
    Post(name="test_public", is_private=False, timestamp=1).save()
    Post(name="test_private", is_private=True, timestamp=1).save()
    Post(name="test_private_2", is_private=True, timestamp=1).save()

    request = rf.get("/")
    request.user = "test_user"

    Oso.load_str('allow(_, "fail", post: test_app::Post) if post matches {x: 1};')

    with pytest.raises(UnsupportedError):
        q = Post.objects.authorize(request, action="fail")

    # No rules for this.
    q = Post.objects.authorize(request, action="get")
    assert q.count() == 0
