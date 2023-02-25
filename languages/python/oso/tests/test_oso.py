"""Tests the Polar API as an external consumer"""

from pathlib import Path

import pytest

from oso import Oso
from polar import exceptions

# Fake global actor name → company ID map.
# Should be an external database lookup.
actors = {"guest": "1", "president": "1"}


class User:
    name: str = ""
    verified: bool = False

    def __init__(self, name=""):
        self.name = name
        self.verified = False

    def companies(self):
        yield Company(id="0")  # fake, will fail
        yield Company(id=actors[self.name])  # real, will pass


class Widget:
    # Data fields.
    id: str = ""

    # Class variables.
    actions = ("read", "create")

    def __init__(self, id):
        self.id = id

    def company(self):
        return Company(id=self.id)


class Company:
    # Class variables.
    roles = ("guest", "admin")

    def __init__(self, id, default_role=""):
        self.id = id
        self.default_role = default_role

    def role(self, actor: User):
        if actor.name == "president":
            return "admin"
        else:
            return "guest"

    def __eq__(self, other):
        return self.id == other.id


test_oso_file = Path(__file__).parent / "test_oso.polar"


@pytest.fixture
def test_oso():
    oso = Oso()
    oso.register_class(User, name="test_oso::User")
    oso.register_class(Widget, name="test_oso::Widget")
    oso.register_class(Company, name="test_oso::Company")
    oso.register_class(Foo)
    oso.register_class(Bar)
    oso.load_file(test_oso_file)

    return oso


def test_sanity(test_oso):
    pass


class Foo:
    def __init__(self, foo):
        self.foo = foo


class Bar(Foo):
    def __init__(self, bar):
        super()
        self.bar = bar


def test_is_allowed(test_oso):
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "read"
    assert test_oso.is_allowed(actor, action, resource)
    assert test_oso.is_allowed({"username": "guest"}, action, resource)
    assert test_oso.is_allowed("guest", action, resource)

    actor = User(name="president")
    action = "create"
    resource = Company(id="1")
    assert test_oso.is_allowed(actor, action, resource)
    assert test_oso.is_allowed({"username": "president"}, action, resource)


def test_query_rule(test_oso):
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "read"
    assert list(test_oso.query_rule("allow", actor, action, resource))


def test_fail(test_oso):
    actor = User(name="guest")
    resource = Widget(id="1")
    action = "not_allowed"
    assert not test_oso.is_allowed(actor, action, resource)
    assert not test_oso.is_allowed({"username": "guest"}, action, resource)


def test_instance_from_external_call(test_oso):
    user = User(name="guest")
    resource = Company(id="1")
    assert test_oso.is_allowed(user, "frob", resource)
    assert test_oso.is_allowed({"username": "guest"}, "frob", resource)


def test_allow_model(test_oso):
    """Test user auditor can list companies but not widgets"""
    user = User(name="auditor")
    assert not test_oso.is_allowed(user, "list", Widget)
    assert test_oso.is_allowed(user, "list", Company)


def test_get_allowed_actions(test_oso):
    test_oso.clear_rules()

    with open(test_oso_file, "rb") as f:
        policy = f.read().decode("utf-8")

        policy1 = (
            policy
            + """allow(_actor: test_oso::User{name: "Sally"}, action, _resource: test_oso::Widget{id: "1"}) if
        action in ["CREATE", "UPDATE"];"""
        )
        test_oso.load_str(policy1)
        user = User(name="Sally")
        resource = Widget(id="1")
        assert set(test_oso.get_allowed_actions(user, resource)) == {
            "read",
            "CREATE",
            "UPDATE",
        }

        test_oso.clear_rules()

        policy2 = (
            policy
            + """allow(_actor: test_oso::User{name: "John"}, _action, _resource: test_oso::Widget{id: "1"});"""
        )
        test_oso.load_str(policy2)
        user = User(name="John")
        with pytest.raises(exceptions.OsoError):
            test_oso.get_allowed_actions(user, resource)
        assert set(
            test_oso.get_allowed_actions(user, resource, allow_wildcard=True)
        ) == {"*"}


def test_forall_with_dot_lookup_and_method_call():
    """Thanks to user Alex Pearce for this test case!"""
    import uuid
    from dataclasses import dataclass, field
    from typing import List

    from oso import ForbiddenError, NotFoundError, Oso

    @dataclass(frozen=True)
    class User:
        name: str
        scopes: List[str]
        id: str = field(default_factory=lambda: str(uuid.uuid4()))

        def has_scope(self, scope: str):
            print(f"Checking scope {scope}")
            return scope in self.scopes

    # Placeholder for a Flask/Starlette Request object
    @dataclass(frozen=True)
    class Request:
        # The scopes defined on the route
        # A token must have these scopes to be access to access the route
        scopes: List[str] = field(default_factory=list)

    def check_request(actor, request):
        """Helper to convert an Oso exception to a True/False decision."""
        try:
            oso.authorize_request(actor, request)
        except (ForbiddenError, NotFoundError):
            return False
        return True

    def expect(value, expected):
        assert value == expected

    oso = Oso()
    oso.clear_rules()
    oso.register_class(User)
    oso.register_class(Request)
    oso.load_str(
        """
# allow(actor: Actor, action: String, resource: Resource) if
#    has_permission(actor, action, resource);
allow(_, _, _);

# A Token is authorised if has all scopes required by the route being accessed
# in the request
allow_request(user: User, request: Request) if
  request_scopes = request.scopes and
  forall(scope in request_scopes, user.has_scope(scope));
  # forall(scope in request.scopes, scope in user.scopes);
    """
    )

    # Org owner
    user = User(name="Dave", scopes=["xyz"])
    # A request with no scopes
    expect(check_request(user, Request()), True)
    # A request with scopes the User has
    expect(check_request(user, Request(scopes=["xyz"])), True)
    # A request with scopes the User has
    expect(check_request(user, Request(scopes=["xyzxyz"])), False)


if __name__ == "__main__":
    pytest.main([__file__])
