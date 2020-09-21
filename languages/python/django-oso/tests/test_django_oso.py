from pathlib import Path
import sys

import pytest

from django.conf import settings
from django.test import RequestFactory
from django.core.exceptions import PermissionDenied

from django_oso.oso import Oso, reset_oso
from django_oso.auth import authorize

from oso import OsoError


@pytest.fixture(autouse=True)
def _reset_oso():
    reset_oso()


@pytest.fixture
def simple_policy():
    """Load simple authorization policy."""
    Oso.load_file(Path(__file__).parent / "simple.polar")


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
