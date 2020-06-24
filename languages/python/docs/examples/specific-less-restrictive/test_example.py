import pytest

from oso import Oso
from polar import Variable

import policy

class Actor:
    def __init__(self, num):
        self.num = num


@pytest.fixture
def oso():
    oso_ = Oso()
    oso_.load_file("policy.polar")
    policy.setup(oso_)

    return oso_

@pytest.mark.xfail(reason="This doesn't work with allow (cut needed).")
def test_specific_allow(oso):
    # The specific resource should only allow 2
    actor_one = Actor(1)
    actor_two = Actor(2)

    specific = policy.Specific()

    assert not oso.allow(actor_one, "get", specific)
    assert oso.allow(actor_two, "get", specific)

def test_specific_decide(oso):
    # The specific resource should only allow 2
    actor_one = Actor(1)
    actor_two = Actor(2)

    specific = policy.Specific()

    assert not oso.query_predicate("decide", actor_one, "get",
                                   specific,
                                   Variable("result")).results[0]['result'] == 'allow'
    assert oso.query_predicate("decide", actor_two, "get", specific,
                               Variable("result")).results[0]['result'] == 'allow'
