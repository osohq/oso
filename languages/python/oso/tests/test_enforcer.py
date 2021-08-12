"""Tests the Enforcement API"""

from oso.exceptions import AuthorizationError, ForbiddenError, NotFoundError
from oso import Enforcer
from pathlib import Path
import pytest

from oso import Oso, polar_class
from polar import exceptions

# Fake global actor name → company ID map.
# Should be an external database lookup.
actors = {"guest": "1", "president": "1"}


class Actor:
    name: str = ""

    def __init__(self, name=""):
        self.name = name

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

    def role(self, actor: Actor):
        if actor.name == "president":
            return "admin"
        else:
            return "guest"

    def __eq__(self, other):
        return self.id == other.id


@pytest.fixture
def test_enforcer():
    oso = Oso()
    oso.register_class(Actor, name="test_oso::Actor")
    oso.register_class(Widget, name="test_oso::Widget")
    oso.register_class(Company, name="test_oso::Company")
    oso.load_file(Path(__file__).parent / "test_oso.polar")

    return Enforcer(oso)


def test_authorize(test_enforcer):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "read"
    test_enforcer.authorize(actor, action, resource)
    test_enforcer.authorize({"username": "guest"}, action, resource)
    test_enforcer.authorize("guest", action, resource)

    actor = Actor(name="president")
    action = "create"
    resource = Company(id="1")
    test_enforcer.authorize(actor, action, resource)
    test_enforcer.authorize({"username": "president"}, action, resource)


def test_fail_authorize(test_enforcer):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "not_allowed"
    # ForbiddenError is expected because actor can "read" resource
    with pytest.raises(ForbiddenError):
        test_enforcer.authorize(actor, action, resource)
    with pytest.raises(ForbiddenError):
        test_enforcer.authorize({"username": "guest"}, action, resource)
    # NotFoundError is expected because actor can NOT "read" resource
    resource = Company(id="1")
    with pytest.raises(NotFoundError):
        test_enforcer.authorize({"username": "guest"}, action, resource)


def test_authorized_actions(test_enforcer):
    rule = """allow(_actor: test_oso::Actor{name: "Sally"}, action, _resource: test_oso::Widget{id: "1"}) if
        action in ["CREATE", "READ"];"""

    test_enforcer.oso.load_str(rule)
    user = Actor(name="Sally")
    resource = Widget(id="1")
    assert set(test_enforcer.authorized_actions(user, resource)) == set(
        ["read", "CREATE", "READ"]
    )

    rule = """allow(_actor: test_oso::Actor{name: "John"}, _action, _resource: test_oso::Widget{id: "1"});"""
    test_enforcer.oso.load_str(rule)
    user = Actor(name="John")
    with pytest.raises(exceptions.OsoError):
        test_enforcer.authorized_actions(user, resource)
    assert set(
        test_enforcer.authorized_actions(user, resource, allow_wildcard=True)
    ) == set(["*"])


def test_authorize_request(test_enforcer):
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

    test_enforcer.oso.register_class(Request)
    test_enforcer.oso.load_str(policy)

    test_enforcer.authorize_request("graham", Request("GET", "/repos/1"))
    with pytest.raises(ForbiddenError):
        test_enforcer.authorize_request("sam", Request("GET", "/repos/1"))

    test_enforcer.authorize_request(verified, Request("GET", "/account"))
    with pytest.raises(ForbiddenError):
        test_enforcer.authorize_request("graham", Request("GET", "/account"))


def test_custom_errors():
    class TestException(Exception):
        def __init__(self, *args):
            self.args = args
    oso = Oso()
    enforcer = Enforcer(oso, get_error=lambda *args: TestException(*args))
    with pytest.raises(TestException) as excinfo:
        enforcer.authorize("graham", "frob", "bar")
    assert excinfo.value.args == (True, "graham", "frob", "bar")


def test_custom_read_action():
    oso = Oso()
    enforcer = Enforcer(oso, read_action="fetch")
    with pytest.raises(AuthorizationError) as excinfo:
        enforcer.authorize("graham", "frob", "bar")
    assert excinfo.type == NotFoundError

    # Allow user to "fetch" bar
    oso.load_str("""allow("graham", "fetch", "bar");""")
    with pytest.raises(AuthorizationError) as excinfo:
        enforcer.authorize("graham", "frob", "bar")
    assert excinfo.type == ForbiddenError


if __name__ == "__main__":
    pytest.main([__file__])
