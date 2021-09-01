"""Tests the Enforcement API"""

from oso.exceptions import ForbiddenError, NotFoundError
from pathlib import Path
import pytest

from oso import Oso
from polar import exceptions
from .test_oso import Actor, Widget, Company


@pytest.fixture
def test_oso():
    oso = Oso()
    oso.register_class(Actor, name="test_oso::Actor")
    oso.register_class(Widget, name="test_oso::Widget")
    oso.register_class(Company, name="test_oso::Company")
    oso.load_file(Path(__file__).parent / "test_oso.polar")

    return oso


def test_authorize(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "read"
    test_oso.authorize(actor, action, resource)
    test_oso.authorize({"username": "guest"}, action, resource)
    test_oso.authorize("guest", action, resource)

    actor = Actor(name="president")
    action = "create"
    resource = Company(id="1")
    test_oso.authorize(actor, action, resource)
    test_oso.authorize({"username": "president"}, action, resource)


def test_fail_authorize(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "not_allowed"
    # ForbiddenError is expected because actor can "read" resource
    with pytest.raises(ForbiddenError):
        test_oso.authorize(actor, action, resource)
    with pytest.raises(ForbiddenError):
        test_oso.authorize({"username": "guest"}, action, resource)
    # NotFoundError is expected because actor can NOT "read" resource
    resource = Company(id="1")
    with pytest.raises(NotFoundError):
        test_oso.authorize({"username": "guest"}, action, resource)


def test_authorized_actions(test_oso):
    rule = """allow(_actor: test_oso::Actor{name: "Sally"}, action, _resource: test_oso::Widget{id: "1"}) if
        action in ["CREATE", "UPDATE"];"""

    test_oso.load_str(rule)
    user = Actor(name="Sally")
    resource = Widget(id="1")
    assert test_oso.authorized_actions(user, resource) == {"read", "CREATE", "UPDATE"}

    rule = """allow(_actor: test_oso::Actor{name: "John"}, _action, _resource: test_oso::Widget{id: "1"});"""
    test_oso.load_str(rule)
    user = Actor(name="John")
    with pytest.raises(exceptions.OsoError):
        test_oso.authorized_actions(user, resource)
    assert test_oso.authorized_actions(user, resource, allow_wildcard=True) == {"*"}


def test_authorize_request(test_oso):
    class Request:
        def __init__(self, method, path) -> None:
            self.method = method
            self.path = path

    policy = """
    allow_request("graham", request: Request) if
        request.path.startswith("/repos");

    allow_request(user: test_oso::Actor, request: Request) if
        request.path.startswith("/account")
        and user.verified;
    """

    verified = Actor("verified")
    verified.verified = True

    test_oso.register_class(Request)
    test_oso.load_str(policy)

    test_oso.authorize_request("graham", Request("GET", "/repos/1"))
    with pytest.raises(ForbiddenError):
        test_oso.authorize_request("sam", Request("GET", "/repos/1"))

    test_oso.authorize_request(verified, Request("GET", "/account"))
    with pytest.raises(ForbiddenError):
        test_oso.authorize_request("graham", Request("GET", "/account"))


def test_authorize_field(test_oso):
    admin = Actor(name="president")
    guest = Actor(name="guest")
    company = Company(id="1")
    resource = Widget(id=company.id)
    # Admin can update name
    test_oso.authorize_field(admin, "update", resource, "name")
    # Admin cannot update another field
    with pytest.raises(ForbiddenError):
        test_oso.authorize_field(guest, "update", resource, "foo")

    # Guests can read non-private fields
    test_oso.authorize_field(guest, "read", resource, "name")
    with pytest.raises(ForbiddenError):
        test_oso.authorize_field(guest, "read", resource, "private_field")


def test_authorized_fields(test_oso):
    admin = Actor(name="president")
    guest = Actor(name="guest")
    company = Company(id="1")
    resource = Widget(id=company.id)
    # Admin should be able to update all fields
    assert test_oso.authorized_fields(admin, "update", resource) == {
        "name",
        "purpose",
        "private_field",
    }
    # Guests should not be able to update fields
    assert test_oso.authorized_fields(guest, "update", resource) == set()
    # Admins should be able to read all fields
    assert test_oso.authorized_fields(admin, "read", resource) == {
        "name",
        "purpose",
        "private_field",
    }
    # Guests should be able to read all public fields
    assert test_oso.authorized_fields(guest, "read", resource) == {"name", "purpose"}


def test_custom_errors():
    class TestNotFound(Exception):
        pass

    class TestForbidden(Exception):
        pass

    oso = Oso(not_found_error=lambda: TestNotFound, forbidden_error=TestForbidden)
    oso.load_str("""allow("graham", "read", "bar");""")

    with pytest.raises(TestForbidden):
        oso.authorize("graham", "frob", "bar")
    with pytest.raises(TestNotFound):
        oso.authorize("sam", "frob", "bar")


def test_custom_read_action():
    oso = Oso(read_action="fetch")
    with pytest.raises(NotFoundError):
        oso.authorize("graham", "frob", "bar")

    # Allow user to "fetch" bar
    oso.load_str("""allow("graham", "fetch", "bar");""")
    with pytest.raises(ForbiddenError):
        oso.authorize("graham", "frob", "bar")


if __name__ == "__main__":
    pytest.main([__file__])
