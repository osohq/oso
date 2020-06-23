"""Tests the Polar API as an external consumer"""

from authlib.jose import jwt
from contextlib import contextmanager
from flask import Flask, request, Response, g
from pathlib import Path
import pprint
import pytest
from dataclasses import dataclass

import oso
from oso import Oso, polar_class
from oso.jwt import Jwt
from polar import api
from polar.api import Polar, Predicate
from polar.test_helpers import public_key, private_key

# Fake global actor name → company ID map.
# Should be an external database lookup.
actors = {
    "guest": "1",
    "president": "1",
}


@polar_class
class Actor:
    name: str = ""

    def __init__(self, name: ""):
        self.name = name

    def company(self):
        yield Company(id="0")  # fake, will fail
        yield Company(id=actors[self.name])  # real, will pass


# example of registering a non-dataclass class
@polar_class(from_polar="from_polar")
class Widget:
    # Data fields.
    id: str = ""

    # Class variables.
    actions = ("get", "create")

    def __init__(self, id):
        self.id = id

    def company(self):
        return Company(id=self.id)

    def from_polar(id):
        return Widget(id)


@dataclass
@polar_class
class Company:
    # Data fields.
    id: str = ""
    default_role: str = ""

    # Class variables.
    roles = ("guest", "admin")

    def role(self, actor: Actor):
        if actor.name == "president":
            yield "admin"
        else:
            yield "guest"

    def from_polar(id, default_role):
        return Company(id, default_role)


@pytest.fixture(scope="module")
def test_oso():
    _oso = Oso()
    _oso.register_class(Jwt)
    # import the test policy
    _oso.load_file(Path(__file__).parent / "test_oso.polar")

    return _oso


def test_sanity(test_oso):
    pass


def test_decorators(test_oso):
    actor = Actor(name="president")
    action = "create"
    resource = Company(id="1")
    assert test_oso._query_pred(
        Predicate(name="allow", args=(actor, action, resource))
    ).success


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


def test_allow(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert test_oso.allow(actor, action, resource)
    assert test_oso.allow({"username": "guest"}, action, resource)
    assert test_oso.allow("guest", action, resource)
    Jwt.add_key(public_key)
    assert test_oso.allow(token("guest"), action, resource)

    actor = Actor(name="president")
    action = "create"
    resource = Company(id="1")
    assert test_oso.allow(actor, action, resource)
    assert test_oso.allow({"username": "president"}, action, resource)
    assert test_oso.allow(token("president"), action, resource)


def test_query_predicate(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "get"
    assert test_oso.query_predicate("allow", actor, action, resource).success


def test_fail(test_oso):
    actor = Actor(name="guest")
    resource = Widget(id="1")
    action = "not_allowed"
    assert not test_oso.allow(actor, action, resource)
    assert not test_oso.allow({"username": "guest"}, action, resource)
    Jwt.add_key(public_key)
    assert not test_oso.allow(token("guest"), action, resource)


def test_instance_from_external_call(test_oso):
    user = Actor(name="guest")
    resource = Company(id="1")
    assert test_oso.allow(user, "frob", resource)
    assert test_oso.allow({"username": "guest"}, "frob", resource)
    Jwt.add_key(public_key)
    assert test_oso.allow(token("guest"), "frob", resource)


if __name__ == "__main__":
    pytest.main([__file__])
