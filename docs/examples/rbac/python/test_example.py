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
    ["01-simple.polar", "02-simple.polar", "05-external.polar", "06-external.polar",],
)
def test_parses(oso, policy, load):
    class User:
        def __init__(self, role=None, name=None):
            self.role = role
            self.name = name

    oso.register_class(User)

    # Test that policy parses and inline tests pass.
    load(policy)
    oso._load_queued_files()


def test_external_policy(oso, load):
    load("05-external.polar")

    class User:
        def __init__(self, role=None, name=None):
            self.role = role
            self.name = name

    oso.register_class(User)

    oso._load_queued_files()

    assert oso.allow(User(role="employee"), "submit", "expense")
    assert oso.allow(User(role="admin"), "approve", "expense")
    assert not oso.allow(User(role="employee"), "approve", "expense")
    assert oso.allow(User(role="accountant"), "view", "expense")
    assert oso.allow(User(name="greta"), "approve", "expense")


def test_external_policy(oso, load):
    load("06-external.polar")

    class User:
        def __init__(self, role=None, name=None):
            self.role = role
            self.name = name

    oso.register_class(User)

    oso._load_queued_files()

    assert oso.allow(User(role="employee"), "submit", "expense")
    assert not oso.allow(User(role="employee"), "view", "expense")
    assert not oso.allow(User(role="employee"), "approve", "expense")

    assert oso.allow(User(role="accountant"), "view", "expense")
    assert oso.allow(User(role="accountant"), "submit", "expense")
    assert not oso.allow(User(role="accountant"), "approve", "expense")

    assert oso.allow(User(role="admin"), "submit", "expense")
    assert oso.allow(User(role="admin"), "view", "expense")
    assert oso.allow(User(role="admin"), "approve", "expense")
