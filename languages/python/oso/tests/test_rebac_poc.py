import pytest

from pathlib import Path
from oso import Oso, OsoError, Variable


class User:
    pass


class Org:
    pass


def test_rebac_validation():
    o = Oso()
    o.register_actor(User)
    o.register_resource(Org)
    o.load_file(Path(__file__).parent / "rebac_poc.polar")
    o.query_rule("has_role", Variable("user"), Variable("role"), Variable("resource"))
