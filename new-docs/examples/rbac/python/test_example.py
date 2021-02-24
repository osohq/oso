from pathlib import Path

import pytest

from oso import Oso


@pytest.fixture
def oso():
    return Oso()


@pytest.fixture
def load(oso):
    def load(policy):
        oso.load_file(Path(__file__).parent.parent / policy)

    return load


@pytest.mark.parametrize(
    "policy",
    [
        "01-simple.polar",
        "02-simple.polar",
        "05-external.polar",
        "06-external.polar",
    ],
)
def test_parses(oso, policy, load):
    class User:
        def __init__(self, role=None, name=None):
            self.role = role
            self.name = name

    oso.register_class(User)

    # Test that policy parses and inline tests pass.
    load(policy)


def test_external_policy(oso, load):
    load("05-external.polar")

    class User:
        def __init__(self, role=None, name=None):
            self.role = role
            self.name = name

    oso.register_class(User)

    assert oso.is_allowed(User(role="employee"), "submit", "expense")
    assert oso.is_allowed(User(role="admin"), "approve", "expense")
    assert not oso.is_allowed(User(role="employee"), "approve", "expense")
    assert oso.is_allowed(User(role="accountant"), "view", "expense")
    assert oso.is_allowed(User(name="greta"), "approve", "expense")


def test_external_policy(oso, load):
    load("06-external.polar")

    class User:
        def __init__(self, role=None, name=None):
            self.role = role
            self.name = name

    oso.register_class(User)

    assert oso.is_allowed(User(role="employee"), "submit", "expense")
    assert not oso.is_allowed(User(role="employee"), "view", "expense")
    assert not oso.is_allowed(User(role="employee"), "approve", "expense")

    assert oso.is_allowed(User(role="accountant"), "view", "expense")
    assert oso.is_allowed(User(role="accountant"), "submit", "expense")
    assert not oso.is_allowed(User(role="accountant"), "approve", "expense")

    assert oso.is_allowed(User(role="admin"), "submit", "expense")
    assert oso.is_allowed(User(role="admin"), "view", "expense")
    assert oso.is_allowed(User(role="admin"), "approve", "expense")
