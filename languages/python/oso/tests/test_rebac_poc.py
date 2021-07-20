import pytest

from pathlib import Path
from oso import Oso, OsoError, Variable
from polar import Expression, Pattern, Predicate
from polar.exceptions import UnsupportedError

# TODO: move this into the lib, only here for tests to pass


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
    # TODO: think about better way of defining these methods/properties--they're
    # just relationships but need to know if one to many or many to many
    o.register_actor(User, methods={"has_role": bool}, properties={"teams": Team})
    o.register_group(Team, methods={"has_role": bool})
    o.register_resource(Org)
    o.register_resource(Repo, properties={"org": Org})
    o.register_resource(Issue, properties={"repo": Repo})
    o.load_file(Path(__file__).parent / "rebac_poc.polar")
    validate_has_role(o)


def test_validate_has_role():
    # TODO: validate that all constraints described in functional spec are enforced
    raise ("Unimplemented!")


def test_validate_has_permission():
    # TODO: validate that all constraints described in functional spec are enforced
    raise ("Unimplemented!")


def validate_has_role(o):
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
        actor_constraints = b["actor"]
        # actor type should always be abstract
        assert type(actor_constraints) == Expression
        actor_cls_name = get_specializer_tag(actor_constraints)
        assert (actor_cls_name in o.host.actors.keys()) or (
            actor_cls_name in o.host.groups.keys()
        )
        validate_lookups(o, actor_cls_name, actor_constraints)

        role = b["role"]
        if type(role) == Expression:
            assert get_specializer_tag(role) == "String"
        else:
            assert type(role) == str

        resource_constraints = b["resource"]
        # resource type should always be abstract
        assert type(resource_constraints) == Expression
        resource_cls_name = get_specializer_tag(resource_constraints)
        assert resource_cls_name in o.host.resources.keys()
        validate_lookups(o, resource_cls_name, resource_constraints)


def validate_lookups(oso, entity_cls_name, constraints):
    lookups = find_pattern(constraints, "Dot")
    for l in lookups:
        if l.args[0] == Variable("_this"):
            if type(l.args[1]) == Predicate:
                name = l.args[1].name
                args = l.args[1].args
                assert name in oso.host.methods[entity_cls_name].keys()
            elif type(l.args[1]) == str:
                assert l.args[1] in oso.host.properties[entity_cls_name].keys()


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


def test_bug():
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Org)
    rule = """has_role(user: User, "owner", org: Org) if org.user = user;"""
    oso.load_str(rule)
    bindings = next(
        oso.query_rule(
            "has_role",
            Variable("user"),
            "owner",
            Variable("org"),
            accept_expression=True,
        )
    )["bindings"]
    print(bindings["user"])
    print(bindings["org"])