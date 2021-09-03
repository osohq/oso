import pytest

from sqlalchemy import create_engine
from sqlalchemy.types import String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from oso import Oso
from .polar_roles_sqlalchemy_helpers import (
    resource_role_class,
    assign_role,
    remove_role,
)


Base = declarative_base(name="RoleBase")


class Org(Base):  # type: ignore
    __tablename__ = "orgs"

    name = Column(String(), primary_key=True)

    def __repr__(self):
        return f"Org({self.name})"


class User(Base):  # type: ignore
    __tablename__ = "users"

    name = Column(String(), primary_key=True)

    def __repr__(self):
        return f"User({self.name})"


class Repo(Base):  # type: ignore
    __tablename__ = "repos"

    name = Column(String(256), primary_key=True)

    org_name = Column(String, ForeignKey("orgs.name"))
    org = relationship("Org", backref="repos", lazy=True)  # type: ignore

    def __repr__(self):
        return f"Repo({self.name}) <- {self.org}"


class Issue(Base):  # type: ignore
    __tablename__ = "issues"

    name = Column(String(256), primary_key=True)
    repo_name = Column(String(256), ForeignKey("repos.name"))
    repo = relationship("Repo", backref="issues", lazy=True)  # type: ignore

    def __repr__(self):
        return f"Issue({self.name}) <- {self.repo}"


RepoRoleMixin = resource_role_class(User, Repo, ["reader", "writer"])


class RepoRole(Base, RepoRoleMixin):  # type: ignore
    pass


OrgRoleMixin = resource_role_class(User, Org, ["owner", "member"])


class OrgRole(Base, OrgRoleMixin):  # type: ignore
    pass


@pytest.fixture
def init_oso():
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)

    Session = sessionmaker(bind=engine)
    session = Session()

    oso = Oso()

    for m in Base.registry.mappers:
        oso.register_class(m.class_)

    return (oso, session)


@pytest.fixture
def sample_data(init_oso):
    _, session = init_oso

    apple = Org(name="apple")
    osohq = Org(name="osohq")

    ios = Repo(name="ios", org=apple)
    oso_repo = Repo(name="oso", org=osohq)
    demo_repo = Repo(name="demo", org=osohq)

    ios_laggy = Issue(name="laggy", repo=ios)
    oso_bug = Issue(name="bug", repo=oso_repo)

    leina = User(name="leina")
    steve = User(name="steve")
    gabe = User(name="gabe")

    objs = {
        "leina": leina,
        "steve": steve,
        "gabe": gabe,
        "apple": apple,
        "osohq": osohq,
        "ios": ios,
        "oso_repo": oso_repo,
        "demo_repo": demo_repo,
        "ios_laggy": ios_laggy,
        "oso_bug": oso_bug,
    }
    for obj in objs.values():
        session.add(obj)
    session.commit()

    return objs


def test_resource_blocks_with_sqlalchemy_mixins(init_oso, sample_data):
    oso, session = init_oso

    policy = """
      allow(actor, action, resource) if
        has_permission(actor, action, resource);

      has_role(user: User, name, repo: Repo) if
        role in user.repo_roles and
        role.name = name and
        role.resource = repo;

      has_role(user: User, name, org: Org) if
        role in user.org_roles and
        role.name = name and
        role.resource = org;

      actor User {}

      resource Org {
        roles = [ "owner", "member" ];
        permissions = [ "invite", "create_repo" ];

        "create_repo" if "member";
        "invite" if "owner";

        "member" if "owner";
      }

      resource Repo {
        roles = [ "writer", "reader" ];
        permissions = [ "push", "pull" ];
        relations = { parent: Org };

        "pull" if "reader";
        "push" if "writer";

        "reader" if "writer";

        "reader" if "member" on "parent";
        "writer" if "owner" on "parent";
      }

      has_relation(org: Org, "parent", repo: Repo) if
        org = repo.org;

      resource Issue {
        permissions = [ "edit" ];
        relations = { parent: Repo };

        "edit" if "writer" on "parent";
      }

      has_relation(repo: Repo, "parent", issue: Issue) if
        repo = issue.repo;
    """
    oso.load_str(policy)

    # Get sample data
    # -------------------
    leina = sample_data["leina"]
    steve = sample_data["steve"]
    gabe = sample_data["gabe"]

    osohq = sample_data["osohq"]
    # apple = sample_data["apple"]

    oso_repo = sample_data["oso_repo"]
    # ios = sample_data["ios"]
    # demo_repo = sample_data["demo_repo"]

    ios_laggy = sample_data["ios_laggy"]
    oso_bug = sample_data["oso_bug"]

    # @NOTE: Need the users and resources in the db before assigning roles
    # so you have to call session.commit() first.
    assign_role(leina, osohq, "owner", session=session)
    assign_role(steve, osohq, "member", session=session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "create_repo", osohq)
    assert oso.is_allowed(leina, "push", oso_repo)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "edit", oso_bug)

    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "create_repo", osohq)
    assert not oso.is_allowed(steve, "push", oso_repo)
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    assert not oso.is_allowed(leina, "edit", ios_laggy)
    assert not oso.is_allowed(steve, "edit", ios_laggy)

    # oso.actor = leina
    # oso.checked_permissions = {Repo: "pull"}
    # auth_session = auth_sessionmaker()

    # results = auth_session.query(Repo).all()
    # assert len(results) == 2
    # result_ids = [repo.id for repo in results]
    # assert oso_repo.id in result_ids
    # assert demo_repo.id in result_ids
    # assert ios.id not in result_ids

    # oso.actor = leina
    # oso.checked_permissions = {Issue: "edit"}
    # auth_session = auth_sessionmaker()

    # results = auth_session.query(Issue).all()
    # assert len(results) == 1
    # result_ids = [issue.id for issue in results]
    # assert oso_bug.id in result_ids

    assert not oso.is_allowed(gabe, "edit", oso_bug)
    assign_role(gabe, osohq, "member", session=session)
    session.commit()
    assert not oso.is_allowed(gabe, "edit", oso_bug)
    assign_role(gabe, osohq, "owner", session=session)
    session.commit()
    assert oso.is_allowed(gabe, "edit", oso_bug)
    assign_role(gabe, osohq, "member", session=session)
    session.commit()
    assert not oso.is_allowed(gabe, "edit", oso_bug)
    assign_role(gabe, osohq, "owner", session=session)
    session.commit()
    assert oso.is_allowed(gabe, "edit", oso_bug)
    remove_role(gabe, osohq, "owner", session=session)
    session.commit()
    assert not oso.is_allowed(gabe, "edit", oso_bug)
