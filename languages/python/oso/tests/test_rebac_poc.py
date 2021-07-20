import pytest

from pathlib import Path
from oso import Oso, OsoError, Variable
from polar import Expression, Pattern, Predicate
from polar.exceptions import UnsupportedError

# TODO: move this into the lib, only here for tests to pass
class OsoResource:
    pass


class User:
    def __init__(self, teams):
        self.teams = teams

    def has_role(self, role, resource):
        return True


class Team:
    def has_role(self, role, resource):
        return True


class Org:
    pass


class Repo:
    def __init__(self, org):
        self.org = org


class Issue:
    def __init__(self, org):
        self.repo = repo


def test_rebac_validation():
    o = Oso()
    o.register_class(OsoResource)
    o.register_actor(User, methods=["has_role"], properties=["teams"])
    o.register_group(Team, methods=["has_role"])
    o.register_resource(Org)
    o.register_resource(Repo, properties=["org"])
    o.register_resource(Issue, properties=["repo"])
    o.load_file(Path(__file__).parent / "rebac_poc.polar")
    results = []
    results = list(
        o.query_rule(
            "has_role",
            Variable("actor"),
            Variable("role"),
            Variable("resource"),
            accept_expression=True,
            method_constraints=True,
        )
    )

    for res in results:
        b = res["bindings"]
        actor = b["actor"]
        # actor type should always be abstract
        assert type(actor) == Expression
        lookups = find_pattern(actor, "Dot")
        actor_class = get_specializer_tag(actor)
        assert actor_class in o.host.actors.keys()
        for l in lookups:
            assert l.args[0] == Variable("_this")
            if type(l.args[1]) == Predicate:
                name = l.args[1].name
                args = l.args[1].args
                assert name in o.host.methods[actor_class]

        role = b["role"]
        assert type(role) == str

        resource = b["resource"]
        # resource type should always be abstract
        assert type(resource) == Expression
        assert get_specializer_tag(resource) in o.host.resources.keys()


def get_specializer_tag(expr):
    specs = find_pattern(expr, "Isa")
    for spec in specs:
        if spec.args[0] == Variable("_this"):
            assert type(spec.args[1]) == Pattern
            return spec.args[1].tag


def find_pattern(expr, op, args=None):
    if expr.operator == op:
        if args:
            yield args == expr.args
        else:
            yield expr
    for arg in expr.args:
        if type(arg) == Expression:
            res = find_pattern(arg, op, args)
            if res:
                yield from res


# def find_pattern(expr, target_parent_op, pattern, last_parent_op=None):
#     if expr == pattern and last_parent_op == target_parent_op:
#         return True
#     for arg in expr.args:
#         find_pattern(arg, target_parent_op, pattern, last_parent_op=expr.op)
