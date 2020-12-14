# type: ignore

import pytest
import datetime

from sqlalchemy import create_engine
from sqlalchemy.types import Integer, String, DateTime
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso import roles as oso_roles, register_models


Base = declarative_base(name="RoleBase")


class Organization(Base):
    __tablename__ = "organizations"

    id = Column(Integer, primary_key=True)
    name = Column(String())
    base_repo_role = Column(String())

    def repr(self):
        return {"id": self.id, "name": self.name}


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True)
    email = Column(String())

    def repr(self):
        return {"id": self.id, "email": self.email}


class Team(Base):
    __tablename__ = "teams"

    id = Column(Integer, primary_key=True)
    name = Column(String(256))

    # many-to-one relationship with organizations
    organization_id = Column(Integer, ForeignKey("organizations.id"))
    organization = relationship("Organization", backref="teams", lazy=True)

    def repr(self):
        return {"id": self.id, "name": self.name}


class Repository(Base):
    __tablename__ = "repositories"

    id = Column(Integer, primary_key=True)
    name = Column(String(256))

    # many-to-one relationship with organizations
    organization_id = Column(Integer, ForeignKey("organizations.id"))
    organization = relationship("Organization", backref="repositories", lazy=True)

    # time info
    created_date = Column(DateTime, default=datetime.datetime.utcnow)
    updated_date = Column(DateTime, default=datetime.datetime.utcnow)

    def repr(self):
        return {"id": self.id, "name": self.name}


class Issue(Base):
    __tablename__ = "issues"

    id = Column(Integer, primary_key=True)
    name = Column(String(256))
    repository_id = Column(Integer, ForeignKey("repositories.id"))
    repository = relationship("Repository", backref="issues", lazy=True)


RepositoryRoleMixin = oso_roles.resource_role_class(
    Base, User, Repository, ["READ", "TRIAGE", "WRITE", "MAINTAIN", "ADMIN"]
)


class RepositoryRole(Base, RepositoryRoleMixin):
    def repr(self):
        return {"id": self.id, "name": str(self.name)}


OrganizationRoleMixin = oso_roles.resource_role_class(
    Base, User, Organization, ["OWNER", "MEMBER", "BILLING"]
)


class OrganizationRole(Base, OrganizationRoleMixin):
    def repr(self):
        return {"id": self.id, "name": str(self.name)}


TeamRoleMixin = oso_roles.resource_role_class(
    Base, User, Team, ["MAINTAINER", "MEMBER"]
)


class TeamRole(Base, TeamRoleMixin):
    def repr(self):
        return {"id": self.id, "name": str(self.name)}


def load_fixture_data(session):
    john = User(email="john@beatles.com")
    paul = User(email="paul@beatles.com")
    admin = User(email="admin@admin.com")
    mike = User(email="mike@monsters.com")
    sully = User(email="sully@monsters.com")
    ringo = User(email="ringo@beatles.com")
    randall = User(email="randall@monsters.com")
    users = [
        john,
        paul,
        admin,
        mike,
        sully,
        ringo,
        randall,
    ]
    for user in users:
        session.add(user)
    beatles = Organization(name="The Beatles", base_repo_role="READ")
    monsters = Organization(name="Monsters Inc.", base_repo_role="READ")
    organizations = [beatles, monsters]
    for org in organizations:
        session.add(org)
    vocalists = Team(name="Vocalists", organization=beatles)
    percussion = Team(name="Percussion", organization=beatles)
    scarers = Team(name="Scarers", organization=monsters)
    teams = [
        vocalists,
        percussion,
        scarers,
    ]
    for team in teams:
        session.add(team)
    abby_road = Repository(name="Abbey Road", organization=beatles)
    paperwork = Repository(name="Paperwork", organization=monsters)
    repositories = [
        abby_road,
        paperwork,
    ]
    for repo in repositories:
        session.add(repo)
    # TODO: issues
    roles = [
        RepositoryRole(name="READ", repository=abby_road, user=john),
        RepositoryRole(name="READ", repository=abby_road, user=paul),
        RepositoryRole(name="READ", repository=paperwork, user=mike),
        RepositoryRole(name="READ", repository=paperwork, user=sully),
        OrganizationRole(
            name="OWNER",
            organization=beatles,
            user=john,
        ),
        OrganizationRole(
            name="MEMBER",
            organization=beatles,
            user=paul,
        ),
        OrganizationRole(
            name="MEMBER",
            organization=beatles,
            user=ringo,
        ),
        OrganizationRole(
            name="OWNER",
            organization=monsters,
            user=mike,
        ),
        OrganizationRole(
            name="MEMBER",
            organization=monsters,
            user=sully,
        ),
        OrganizationRole(
            name="MEMBER",
            organization=monsters,
            user=randall,
        ),
        TeamRole(name="MEMBER", team=vocalists, user=paul),
        TeamRole(name="MAINTAINER", team=vocalists, user=john),
        TeamRole(name="MAINTAINER", team=percussion, user=ringo),
        TeamRole(name="MEMBER", team=scarers, user=randall),
        TeamRole(name="MAINTAINER", team=scarers, user=sully),
    ]
    for role in roles:
        session.add(role)

    session.commit()


@pytest.fixture
def test_db_session():
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)

    Session = sessionmaker(bind=engine)
    session = Session()

    load_fixture_data(session)

    return session

# # @TODO: These relationship fields aren't working.
# def test_user_resources_relationship_fields(test_db_session):
#     beatles = test_db_session.query(Organization).filter_by(name="The Beatles").first()
#     users = beatles.users
#     assert len(users) == 3
#     assert users[0].email == "john@beatles.com"
#
# def test_resource_users_relationship_fields(test_db_session):
#     john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
#     orgs = john.organizations
#     assert len(orgs) == 1
#     assert orgs[0].name == "The Beatles"

def test_get_user_resources_and_roles(test_db_session):
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    roles = john.organization_roles
    assert len(roles) == 1
    assert roles[0].name == "OWNER"
    assert roles[0].organization.name == "The Beatles"

def test_get_user_roles_for_resource(test_db_session):
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    beatles = test_db_session.query(Organization).filter_by(name="The Beatles").first()
    resource_roles = test_db_session.query(OrganizationRole).filter_by(user=john, organization=beatles).all()
    assert len(resource_roles) == 1
    assert resource_roles[0].name == "OWNER"


def test_get_resource_users_and_roles(test_db_session):
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()
    user_roles = abbey_road.roles
    assert user_roles[0].user.email == "john@beatles.com"
    assert user_roles[0].name == "READ"
    assert user_roles[1].user.email == "paul@beatles.com"
    assert user_roles[0].name == "READ"


def test_get_resource_users_with_role(test_db_session):
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()
    users = test_db_session.query(RepositoryRole).filter_by(repository=abbey_road, name="READ").all()
    assert len(users) == 2
    assert users[0].user.email == "john@beatles.com"
    assert users[1].user.email == "paul@beatles.com"


def test_add_user_role(test_db_session):
    ringo = test_db_session.query(User).filter_by(email="ringo@beatles.com").first()
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()

    roles = test_db_session.query(RepositoryRole).filter_by(user=ringo, repository=abbey_road).all()
    assert len(roles) == 0

    new_role = RepositoryRole(name="READ", repository=abbey_road, user=ringo)
    test_db_session.add(new_role)

    roles = test_db_session.query(RepositoryRole).filter_by(user=ringo, repository=abbey_road).all()
    assert len(roles) == 1
    assert roles[0].name == "READ"

    # with pytest.raises(ValueError):
    #     new_role = RepositoryRole(name="NOT_A_REAL_ROLE", repository=abbey_road, user=ringo)


def test_delete_user_role(test_db_session):
    # Test with explicit role arg
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()

    roles = test_db_session.query(RepositoryRole).filter_by(user=john, repository=abbey_road).all()
    assert len(roles) == 1

    test_db_session.delete(roles[0])

    roles = test_db_session.query(RepositoryRole).filter_by(user=john, repository=abbey_road).all()
    assert len(roles) == 0

    paul = test_db_session.query(User).filter_by(email="paul@beatles.com").first()
    roles = test_db_session.query(RepositoryRole).filter_by(user=paul, repository=abbey_road).all()
    assert len(roles) == 1

    test_db_session.query(RepositoryRole).filter_by(user=paul, repository=abbey_road).delete()

    roles = test_db_session.query(RepositoryRole).filter_by(user=paul, repository=abbey_road).all()
    assert len(roles) == 0

    # # Test trying to delete non-existent role raises exception
    # with pytest.raises(Exception):
    #     oso_roles.delete_user_role(test_db_session, paul, abbey_road, "READ")


def test_reassign_user_role(test_db_session):
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()

    roles = test_db_session.query(RepositoryRole).filter_by(user=john, repository=abbey_road).all()
    assert len(roles) == 1
    assert roles[0].name == "READ"

    test_db_session.query(RepositoryRole).filter_by(user=john, repository=abbey_road).update({"name": "WRITE"})

    roles = test_db_session.query(RepositoryRole).filter_by(user=john, repository=abbey_road).all()
    assert len(roles) == 1
    assert roles[0].name == "WRITE"


def test_set_get_session():
    from sqlalchemy_oso.session import set_get_session
    from oso import Oso

    def get_session():
        engine = create_engine("sqlite://")
        Base.metadata.create_all(engine)

        Session = sessionmaker(bind=engine)
        session = Session()

        load_fixture_data(session)

        return session

    oso = Oso()
    set_get_session(oso, get_session)
    register_models(oso, Base)
    test_str = """get_repo(name: String) if
                    session = OsoSession.get() and
                    repo = session.query(Repository).filter_by(name: name).first() and
                    repo.name = name;
                    """

    oso.load_str(test_str)
    results = oso.query_rule("get_repo", "Abbey Road")
    assert next(results)
    results = oso.query_rule("get_repo", "Abbey Road")
    assert next(results)
