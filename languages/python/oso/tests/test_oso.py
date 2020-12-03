"""Tests the Polar API as an external consumer"""

from pathlib import Path
import pytest

from oso import Oso, polar_class

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
    actions = ("get", "create")

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
def test_oso():
    oso = Oso()
    oso.register_class(Actor, name="test_oso::Actor")
    oso.register_class(Widget, name="test_oso::Widget")
    oso.register_class(Company, name="test_oso::Company")
    oso.load_file(Path(__file__).parent / "test_oso.polar")

    return oso


def test_sanity(test_oso):
    pass


def test_decorators(test_oso):
    assert test_oso.is_allowed(FooDecorated(foo=1), "read", BarDecorated(bar=1))


@polar_class
class FooDecorated:
    def __init__(self, foo):
        self.foo = foo


@polar_class
class BarDecorated(FooDecorated):
    def __init__(self, bar):
        super()
        self.bar = bar


def test_is_allowed(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert test_oso.is_allowed(actor, action, resource)
    assert test_oso.is_allowed({"username": "guest"}, action, resource)
    assert test_oso.is_allowed("guest", action, resource)

    actor = Actor(name="president")
    action = "create"
    resource = Company(id="1")
    assert test_oso.is_allowed(actor, action, resource)
    assert test_oso.is_allowed({"username": "president"}, action, resource)


def test_query_rule(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert list(test_oso.query_rule("allow", actor, action, resource))


def test_fail(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "not_allowed"
    assert not test_oso.is_allowed(actor, action, resource)
    assert not test_oso.is_allowed({"username": "guest"}, action, resource)


def test_instance_from_external_call(test_oso):
    user = Actor(name="guest")
    resource = Company(id="1")
    assert test_oso.is_allowed(user, "frob", resource)
    assert test_oso.is_allowed({"username": "guest"}, "frob", resource)


def test_allow_model(test_oso):
    """ Test user auditor can list companies but not widgets"""
    user = Actor(name="auditor")
    assert not test_oso.is_allowed(user, "list", Widget)
    assert test_oso.is_allowed(user, "list", Company)


if __name__ == "__main__":
    pytest.main([__file__])
