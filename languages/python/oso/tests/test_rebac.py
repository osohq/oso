from dataclasses import dataclass
from typing import Dict, List, Union

import pytest


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

    def __init__(self, name: str):
        self.name = name
        self.roles = {}

    def assign_role(self, resource: Resource, name: str):
        self.roles[resource] = name

    def has_role_for_resource(self, name: str, resource: Resource):
        return self.roles.get(resource) == name


def test_rebac(polar, is_allowed):
    [polar.register_class(c) for c in [User, Org, Repo, Issue]]
    polar.load_file("tests/rebac.polar")

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
    assert is_allowed(leina, "invite", oso_hq)
    assert not is_allowed(leina, "invite", apple)
    assert not is_allowed(steve, "invite", oso_hq)
    assert not is_allowed(steve, "invite", apple)

    # from same-resource implication
    assert is_allowed(leina, "create_repo", oso_hq)
    assert not is_allowed(leina, "create_repo", apple)
    assert is_allowed(steve, "create_repo", oso_hq)
    assert not is_allowed(steve, "create_repo", apple)

    # from child-resource implication
    assert is_allowed(leina, "push", oso_repo)
    assert not is_allowed(leina, "push", ios_repo)
    assert is_allowed(leina, "pull", oso_repo)
    assert not is_allowed(leina, "pull", ios_repo)
    assert not is_allowed(steve, "push", oso_repo)
    assert not is_allowed(steve, "push", ios_repo)
    assert is_allowed(steve, "pull", oso_repo)
    assert not is_allowed(steve, "pull", ios_repo)

    # from cross-resource permission
    assert is_allowed(leina, "edit", stephie_bug)
    assert not is_allowed(leina, "edit", laggy)
    assert not is_allowed(steve, "edit", stephie_bug)
    assert not is_allowed(steve, "edit", laggy)

    # from cross-resource permission over two levels of hierarchy
    assert is_allowed(leina, "delete", stephie_bug)
    assert not is_allowed(leina, "delete", laggy)
    assert not is_allowed(steve, "delete", stephie_bug)
    assert not is_allowed(steve, "delete", laggy)

    # from same-resource implication
    assert is_allowed(gabe, "pull", oso_repo)

    # resource-user relationships
    assert not is_allowed(dave, "delete", stephie_bug)
    assert is_allowed(dave, "delete", dave_bug)
    assert not is_allowed(sam, "delete", laggy)
    assert is_allowed(sam, "delete", stephie_bug)
    assert is_allowed(sam, "delete", dave_bug)
