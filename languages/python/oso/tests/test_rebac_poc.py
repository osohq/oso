import pytest

from pathlib import Path
from oso import Oso, OsoError, Variable
from polar import Expression, Pattern


class User:
    pass


class Org:
    pass


def test_rebac_validation():
    o = Oso()
    o.register_actor(User)
    o.register_resource(Org)
    o.load_file(Path(__file__).parent / "rebac_poc.polar")
    results = list(
        o.query_rule(
            "role",
            Variable("actor"),
            Variable("role"),
            Variable("resource"),
            accept_expression=True,
        )
    )
    for res in results:
        b = res["bindings"]
        actor = b["actor"]
        # actor type should always be abstract
        assert type(actor) == Expression
        assert get_specializer_tag(actor) in o.host.actors.keys()

        role = b["role"]
        assert type(role) == str

        resource = b["resource"]
        # resource type should always be abstract
        assert type(resource) == Expression
        assert get_specializer_tag(resource) in o.host.resources.keys()


def get_specializer_tag(expr):
    if expr.operator == "And" and len(expr.args) == 1:
        expr = expr.args[0]
    assert expr.operator == "Isa"
    spec = expr.args[1]
    assert type(spec) == Pattern
    return spec.tag
