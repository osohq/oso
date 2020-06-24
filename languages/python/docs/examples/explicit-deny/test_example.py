import functools
import pytest

import oso

import policy

@pytest.fixture
def auth():
    oso_ = oso.Oso()
    policy.setup(oso_)
    return functools.partial(policy.auth, oso_)


def test_policy(auth):
    alice = {'name': "Alice"}
    bob = {'name': "Bob"}
    mallory = {'name': "Mallory"}

    allowed = {'name': "allowed"}
    allowed2 = {'name': "allowed2"}
    unknown = {'name': "unknown"}

    assert auth(alice, "a", allowed)[0] is True
    assert auth(alice, "a", allowed2)[0] is True
    assert auth(bob, "a", allowed)[0] is True

    assert auth(mallory, "a", allowed2) == (False, "Actor in blacklist")
    assert auth(mallory, "a", allowed) == (False, "Actor in blacklist")
    assert auth(mallory, "a", unknown) == (False, "Actor in blacklist")

    assert auth(alice, "a", unknown) == (False, "Default deny")
