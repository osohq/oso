"""Tests the Polar API as an external consumer"""

from authlib.jose import jwt
from contextlib import contextmanager
from flask import Flask, request, Response, g
from pathlib import Path
import pprint
import pytest
from dataclasses import dataclass

from oso import Oso, polar_class
from oso.jwt import Jwt
from polar import Polar, Predicate
from polar.test_helpers import public_key, private_key

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


@dataclass
class Company:
    # Data fields.
    id: str = ""
    default_role: str = ""

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


@pytest.fixture(scope="module")
def test_oso():
    oso = Oso()
    oso.register_class(Jwt)
    oso.register_class(Actor)
    oso.register_class(Widget)
    oso.register_class(Company)
    oso.load_file(Path(__file__).parent / "test_oso.polar")

    return oso


def test_sanity(test_oso):
    pass


def test_decorators(test_oso):
    actor = Actor(name="president")
    action = "create"
    resource = Company(id="1")
    assert list(test_oso.query(Predicate(name="allow", args=(actor, action, resource))))


@polar_class
class Foo:
    foo: int = 0


@polar_class
class Bar(Foo):
    bar: int = 1


def token(name):
    header = {"alg": "RS256"}
    payload = {"iss": "somebody", "sub": name}
    token = jwt.encode(header, payload, private_key)
    return token.decode("utf-8")


def test_is_allowed(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert test_oso.is_allowed(actor, action, resource)
    assert test_oso.is_allowed({"username": "guest"}, action, resource)
    assert test_oso.is_allowed("guest", action, resource)
    Jwt.add_key(public_key)
    assert test_oso.is_allowed(token("guest"), action, resource)

    actor = Actor(name="president")
    action = "create"
    resource = Company(id="1")
    assert test_oso.is_allowed(actor, action, resource)
    assert test_oso.is_allowed({"username": "president"}, action, resource)
    assert test_oso.is_allowed(token("president"), action, resource)


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
    Jwt.add_key(public_key)
    assert not test_oso.is_allowed(token("guest"), action, resource)


def test_instance_from_external_call(test_oso):
    user = Actor(name="guest")
    resource = Company(id="1")
    assert test_oso.is_allowed(user, "frob", resource)
    assert test_oso.is_allowed({"username": "guest"}, "frob", resource)
    Jwt.add_key(public_key)
    assert test_oso.is_allowed(token("guest"), "frob", resource)


def test_allow_model(test_oso):
    """ Test user auditor can list companies but not widgets"""
    user = Actor(name="auditor")
    assert not test_oso.is_allowed(user, "list", Widget)
    assert test_oso.is_allowed(user, "list", Company)


if __name__ == "__main__":
    pytest.main([__file__])
