# type: ignore

import pytest

from sqlalchemy import create_engine
from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso import SQLAlchemyOso

# from sqlalchemy_oso.auth import authorize_model

Base = declarative_base(name="RoleBase")


class Organization(Base):
    __tablename__ = "organizations"

    name = Column(String(), primary_key=True)

    def __repr__(self):
        return self.name


class User(Base):
    __tablename__ = "users"

    user_id = Column(Integer, primary_key=True)
    name = Column(String())

    def __repr__(self):
        return self.name


class Repository(Base):
    __tablename__ = "repositories"

    repo_id = Column(Integer, primary_key=True)
    name = Column(String(256))

    # many-to-one relationship with organizations
    organization_id = Column(Integer, ForeignKey("organizations.name"))
    organization = relationship("Organization", backref="repositories", lazy=True)

    def __repr__(self):
        return self.name


class Issue(Base):
    __tablename__ = "issues"

    issue_id = Column(Integer, primary_key=True)
    name = Column(String(256))
    repository_id = Column(Integer, ForeignKey("repositories.repo_id"))
    repository = relationship("Repository", backref="issues", lazy=True)


def test_roles3():
    engine = create_engine("sqlite://", echo=False)
    Session = sessionmaker(bind=engine)

    oso = SQLAlchemyOso(Base)
    oso.enable_roles(User, Session)

    oso.load_file("sqlalchemy_oso/roles.polar")
    oso.load_file("sqlalchemy_oso/roles_demo.polar")

    oso.roles.synchronize_data()  # role_allows rule gets added here probably

    Base.metadata.create_all(engine)

    steve = User(name="steve")
    leina = User(name="leina")
    gabe = User(name="gabe")
    osohq = Organization(name="osohq")
    acme = Organization(name="acme")
    oso_repo = Repository(name="oso", organization=osohq)
    anvil_repo = Repository(name="anvil", organization=acme)

    objs = [steve, leina, gabe, osohq, oso_repo, acme, anvil_repo]

    session = Session()
    for obj in objs:
        session.add(obj)

    # Things that happen in the app via the management api.
    oso.roles.assign_role(leina, osohq, "owner", session)
    oso.roles.assign_role(steve, osohq, "member", session)
    oso.roles.assign_role(gabe, oso_repo, "writer", session)

    # Test

    # Test Org roles
    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)
    # assert not oso.is_allowed(leina, "invite", acme)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Leina can create a repo because she's the OWNER and OWNER implies MEMBER
    assert oso.is_allowed(leina, "create_repo", osohq)

    # Steve can pull from oso_repo because he is a MEMBER of osohq
    # which implies READ on oso_repo
    # oso.register_constant(steve, "steve")
    # oso.register_constant(osohq, "osohq")
    # oso.register_constant(oso_repo, "oso_repo")
    # oso.register_constant(anvil_repo, "anvil_repo")
    # oso.repl()
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert not oso.is_allowed(steve, "pull", anvil_repo)
    # Leina can pull from oso_repo because she's an OWNER of osohq
    # which implies WRITE on oso_repo
    # which implies READ on oso_repo
    assert oso.is_allowed(leina, "pull", oso_repo)
    # Gabe can pull from oso_repo because he has WRTIE on oso_repo
    # which implies READ on oso_repo
    assert oso.is_allowed(gabe, "pull", oso_repo)

    # Steve can NOT push to oso_repo because he is a MEMBER of osohq
    # which implies READ on oso_repo but not WRITE
    assert not oso.is_allowed(steve, "push", oso_repo)
    # Leina can push to oso_repo because she's an OWNER of osohq
    # which implies WRITE on oso_repo
    assert oso.is_allowed(leina, "push", oso_repo)
    # Gabe can push to oso_repo because he has WRTIE on oso_repo
    assert oso.is_allowed(gabe, "push", oso_repo)

    # # Data filtering test:
    # auth_filter = authorize_model(oso, leina, "push", session, Repository)
    # assert str(auth_filter) == "repositories.organization_id = :org_id_1"
