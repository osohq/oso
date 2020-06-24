import pytest
import functools

from oso import Oso

import policy
from policy import OsoModel, OsoModel2, NotOsoModel

@pytest.fixture
def auth():
    oso_ = Oso()
    policy.setup(oso_)

    return functools.partial(policy.auth, oso_)

def test_correct(auth):
    called = object()
    def next(*args):
        return called

    assert auth("a", "a", OsoModel(), next) is True
    assert auth("a", "a", OsoModel2(), next) is False
    assert auth("a", "a", NotOsoModel(), next) is called
