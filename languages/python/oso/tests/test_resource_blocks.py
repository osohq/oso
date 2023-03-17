from dataclasses import dataclass
from typing import Dict, List, Union


class Org:
    owner: "User"
    repos: List["Repo"]

    def __init__(self, *, owner: "User"):
        self.owner = owner
        self.repos = []

    def create_repo(self, *, is_public: bool):
        repo = Repo(org=self, is_public=is_public)
        self.repos.append(repo)
        return repo


class Repo:
    org: Org
    issues: List["Issue"]
    is_public: bool

    def __init__(self, *, org: Org, is_public: bool):
        self.org = org
        self.is_public = is_public
        self.issues = []

    def create_issue(self, *, creator: "User"):
        issue = Issue(repo=self, creator=creator)
        self.issues.append(issue)
        return issue


@dataclass(frozen=True)
class Issue:
    creator: "User"
    repo: Repo


Resource = Union[Org, Repo, Issue]


class BaseActor:
    name: str
    roles: Dict[Resource, str]

    def __init__(self):
        self.roles = {}

    def assign_role(self, *, resource: Resource, name: str):
        self.roles[resource] = name

    def has_role_for_resource(self, *, name: str, resource: Resource):
        return self.roles.get(resource) == name


class User(BaseActor):
    teams: List["Team"]

    def __init__(self, **kwargs):
        self.teams = []
        super().__init__(**kwargs)


class Team(BaseActor):
    users: List[User]

    def __init__(self, **kwargs):
        self.users = []
        super().__init__(**kwargs)

    def add_users(self, *users: User):
        for u in users:
            self.users.append(u)
            u.teams.append(self)
        return self


def test_resource_blocks(polar, is_allowed):
    [polar.register_class(c) for c in [User, Team, Org, Repo, Issue]]
    polar.load_file("tests/resource_blocks.polar")

    annie, dave, gabe, graham, leina, lito, sam, shraddha, stephie, steve, tim = (
        User() for _ in range(11)
    )
    oso_eng_team = Team().add_users(gabe, leina, steve)
    oso_mgr_team = Team().add_users(dave)

    osohq_org = Org(owner=sam)
    apple_org = Org(owner=tim)

    oso_repo = osohq_org.create_repo(is_public=False)
    ios_repo = apple_org.create_repo(is_public=False)
    swift_repo = apple_org.create_repo(is_public=True)

    stephie_issue = oso_repo.create_issue(creator=stephie)
    steve_issue = oso_repo.create_issue(creator=steve)
    shraddha_issue = ios_repo.create_issue(creator=shraddha)

    graham.assign_role(resource=osohq_org, name="owner")
    annie.assign_role(resource=osohq_org, name="member")
    lito.assign_role(resource=oso_repo, name="writer")

    oso_eng_team.assign_role(resource=oso_repo, name="writer")
    oso_mgr_team.assign_role(resource=oso_repo, name="admin")

    # from direct role assignment
    assert is_allowed(graham, "invite", osohq_org)
    assert not is_allowed(graham, "invite", apple_org)
    assert not is_allowed(annie, "invite", osohq_org)
    assert not is_allowed(annie, "invite", apple_org)

    # from same-resource implication
    assert is_allowed(graham, "create_repo", osohq_org)
    assert not is_allowed(graham, "create_repo", apple_org)
    assert is_allowed(annie, "create_repo", osohq_org)
    assert not is_allowed(annie, "create_repo", apple_org)

    # from child-resource implication
    assert is_allowed(graham, "push", oso_repo)
    assert not is_allowed(graham, "push", ios_repo)
    assert is_allowed(graham, "pull", oso_repo)
    assert not is_allowed(graham, "pull", ios_repo)
    assert not is_allowed(annie, "push", oso_repo)
    assert not is_allowed(annie, "push", ios_repo)
    assert is_allowed(annie, "pull", oso_repo)
    assert not is_allowed(annie, "pull", ios_repo)

    # from cross-resource permission
    assert is_allowed(graham, "edit", stephie_issue)
    assert not is_allowed(graham, "edit", shraddha_issue)
    assert not is_allowed(annie, "edit", stephie_issue)
    assert not is_allowed(annie, "edit", shraddha_issue)

    # from cross-resource permission over two levels of hierarchy
    assert is_allowed(graham, "delete", stephie_issue)
    assert not is_allowed(graham, "delete", shraddha_issue)
    assert not is_allowed(annie, "delete", stephie_issue)
    assert not is_allowed(annie, "delete", shraddha_issue)

    # from same-resource implication
    assert is_allowed(lito, "pull", oso_repo)

    # resource-user relationships
    assert not is_allowed(steve, "delete", stephie_issue)
    assert is_allowed(steve, "delete", steve_issue)
    assert not is_allowed(sam, "delete", shraddha_issue)
    assert is_allowed(sam, "delete", stephie_issue)
    assert is_allowed(sam, "delete", steve_issue)

    # pure ABAC
    assert not is_allowed(graham, "pull", ios_repo)
    assert is_allowed(graham, "pull", swift_repo)

    # groups
    assert is_allowed(oso_eng_team, "push", oso_repo)
    assert is_allowed(oso_mgr_team, "push", oso_repo)
    assert not is_allowed(oso_eng_team, "delete", stephie_issue)
    assert is_allowed(oso_mgr_team, "delete", stephie_issue)

    # user implied by membership in group
    assert is_allowed(leina, "push", oso_repo)
    assert is_allowed(dave, "push", oso_repo)
    assert not is_allowed(leina, "delete", stephie_issue)
    assert is_allowed(dave, "delete", stephie_issue)
