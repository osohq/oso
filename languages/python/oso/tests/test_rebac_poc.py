import pytest

from pathlib import Path
from oso import Oso, OsoError, Variable
from polar import Expression, Pattern, Predicate
from polar.exceptions import UnsupportedError
from dataclasses import dataclass

from typing import List, Dict, Union
from dataclasses import dataclass

# TODO: move this into the lib, only here for tests to pass
class Org:
    name: str
    owner: "User"
    repos: List["Repo"]

    def __init__(self, name: str, owner: "User"):
        self.name = name
        self.owner = owner
        self.repos = []

    def create_repo(self, name: str):
        repo = Repo(name=name, org=self)
        self.repos.append(repo)
        return repo


class Repo:
    name: str
    org: Org
    issues: List["Issue"]

    def __init__(self, name: str, org: Org):
        self.name = name
        self.org = org
        self.issues = []

    def create_issue(self, name: str, creator: "User"):
        issue = Issue(name=name, repo=self, created_by=creator)
        self.issues.append(issue)
        return issue


@dataclass(frozen=True)
class Issue:
    name: str
    created_by: "User"
    repo: Repo


Resource = Union[Org, Repo, Issue]


class User:
    name: str
    roles: Dict[Resource, str]
    teams: List["Team"]

    def __init__(self, name: str, teams=[]):
        self.name = name
        self.roles = {}
        self.teams = teams

    def assign_role(self, resource: Resource, name: str):
        self.roles[resource] = name

    def has_role(self, name: str, resource: Resource):
        print("here")
        return self.roles.get(resource) == name


class Team:
    name: str
    roles: Dict[Resource, str]

    def __init__(self, name: str):
        self.name = name
        self.roles = {}

    def assign_role(self, resource: Resource, name: str):
        self.roles[resource] = name

    def has_role(self, name: str, resource: Resource):
        return self.roles.get(resource) == name


# class User:
#     def __init__(self, teams):
#         self.teams = teams

#     def has_role(self, role, resource):
#         return True


# class Org:
#     pass


# class Repo:
#     def __init__(self, org):
#         self.org = org


# class Issue:
#     def __init__(self, org):
#         self.repo = repo


@pytest.fixture()
def init_oso():
    o = Oso()
    o.register_actor(User, methods={"has_role": bool}, properties={"teams": Team})
    o.register_group(Team, methods={"has_role": bool})
    o.register_resource(Org)
    o.register_resource(Repo, properties={"org": Org})
    o.register_resource(Issue, properties={"repo": Repo})
    o.load_file(Path(__file__).parent / "rebac_poc.polar")

    return o


def test_rebac_policy(init_oso):
    o = init_oso

    leina = User("leina")
    gabe = User("gabe")
    steve = User("steve")
    dave = User("dave")
    sam = User("sam")
    tim = User("tim")
    shraddha = User("shraddha")
    stephie = User("stephie")
    oso_hq = Org("OsoHQ", owner=sam)
    apple = Org("Apple", owner=tim)
    oso_repo = Repo(name="oso", org=oso_hq)
    ios_repo = Repo(name="ios", org=apple)
    stephie_bug = Issue(name="stephie_bug", repo=oso_repo, created_by=stephie)
    dave_bug = Issue(name="dave_bug", repo=oso_repo, created_by=dave)
    laggy = Issue(name="laggy", repo=ios_repo, created_by=shraddha)

    leina.assign_role(oso_hq, "owner")
    gabe.assign_role(oso_repo, "writer")
    steve.assign_role(oso_hq, "member")

    # from direct role assignment
    assert o.is_allowed(leina, "invite", oso_hq)
    assert not o.is_allowed(leina, "invite", apple)
    assert not o.is_allowed(steve, "invite", oso_hq)
    assert not o.is_allowed(steve, "invite", apple)

    # from same-resource implication
    assert o.is_allowed(leina, "create_repo", oso_hq)
    assert not o.is_allowed(leina, "create_repo", apple)
    assert o.is_allowed(steve, "create_repo", oso_hq)
    assert not o.is_allowed(steve, "create_repo", apple)

    # from child-resource implication
    assert o.is_allowed(leina, "push", oso_repo)
    assert not o.is_allowed(leina, "push", ios_repo)
    assert o.is_allowed(leina, "pull", oso_repo)
    assert not o.is_allowed(leina, "pull", ios_repo)
    assert not o.is_allowed(steve, "push", oso_repo)
    assert not o.is_allowed(steve, "push", ios_repo)
    assert o.is_allowed(steve, "pull", oso_repo)
    assert not o.is_allowed(steve, "pull", ios_repo)

    # from cross-resource permission
    assert o.is_allowed(leina, "edit", stephie_bug)
    assert not o.is_allowed(leina, "edit", laggy)
    assert not o.is_allowed(steve, "edit", stephie_bug)
    assert not o.is_allowed(steve, "edit", laggy)

    # from cross-resource permission over two levels of hierarchy
    assert o.is_allowed(leina, "delete", stephie_bug)
    assert not o.is_allowed(leina, "delete", laggy)
    assert not o.is_allowed(steve, "delete", stephie_bug)
    assert not o.is_allowed(steve, "delete", laggy)

    # from same-resource implication
    assert o.is_allowed(gabe, "pull", oso_repo)

    # resource-user relationships
    assert not o.is_allowed(dave, "delete", stephie_bug)
    assert o.is_allowed(dave, "delete", dave_bug)
    assert not o.is_allowed(sam, "delete", laggy)
    assert o.is_allowed(sam, "delete", stephie_bug)
    assert o.is_allowed(sam, "delete", dave_bug)


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