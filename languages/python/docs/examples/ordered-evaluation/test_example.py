import pytest

from oso import Oso

import policy
from policy import ComplicatedResource

@pytest.fixture
def oso():
    oso_ = Oso()
    policy.setup(oso_)
    return oso_

def test_ordered_eval(oso):
    actor = lambda name, role: {'name': name, 'role': role}

    # Allowed
    assert oso.allow(actor('Alice', 'normal'), "a", ComplicatedResource())

    # Blocked
    assert not oso.allow(actor('Mallory', 'normal'), "a", ComplicatedResource())
    assert not oso.allow(actor('Mallory', 'superuser'), "a", ComplicatedResource())
    assert not oso.allow(actor('Wallace', 'normal'), "a", ComplicatedResource())

    # Allowed because superuser
    assert oso.allow(actor('Jim', 'superuser'), "a", ComplicatedResource())
    assert not oso.allow(actor('Jim', 'normal'), "a", ComplicatedResource())

    # Resource is unrestricted
    assert oso.allow(actor('Anybody', 'normal'), 'a', ComplicatedResource(True))
    assert not oso.allow(actor('Anybody', 'normal'), 'a', ComplicatedResource())
