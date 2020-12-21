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
    abby_road_read = RepositoryRole(
        name="READ",
        repository=abby_road,
        users=[john, paul],
    )
    abby_road_triage = RepositoryRole(
        name="TRIAGE",
        repository=abby_road,
        users=[],
    )
    abby_road_write = RepositoryRole(
        name="WRITE",
        repository=abby_road,
        users=[],
    )
    abby_road_maintain = RepositoryRole(
        name="MAINTAIN",
        repository=abby_road,
        users=[],
    )
    abby_road_admin = RepositoryRole(
        name="ADMIN",
        repository=abby_road,
        users=[],
    )
    paperwork_read = RepositoryRole(
        name="READ",
        repository=paperwork,
        users=[mike, sully],
    )
    paperwork_triage = RepositoryRole(
        name="TRIAGE",
        repository=paperwork,
        users=[],
    )
    paperwork_write = RepositoryRole(
        name="WRITE",
        repository=paperwork,
        users=[],
    )
    paperwork_maintain = RepositoryRole(
        name="MAINTAIN",
        repository=paperwork,
        users=[],
    )
    paperwork_admin = RepositoryRole(
        name="ADMIN",
        repository=paperwork,
        users=[],
    )
    repo_roles = [
        abby_road_read,
        abby_road_triage,
        abby_road_write,
        abby_road_maintain,
        abby_road_admin,
        paperwork_read,
        paperwork_triage,
        paperwork_write,
        paperwork_maintain,
        paperwork_admin,
    ]
    for repo_role in repo_roles:
        session.add(repo_role)
    beatles_owner = OrganizationRole(
        name="OWNER",
        organization=beatles,
        users=[john],
    )
    beatles_member = OrganizationRole(
        name="MEMBER",
        organization=beatles,
        users=[paul, ringo],
    )
    monsters_owner = OrganizationRole(
        name="OWNER",
        organization=monsters,
        users=[mike],
    )
    monsters_member = OrganizationRole(
        name="MEMBER",
        organization=monsters,
        users=[sully, randall],
    )
    org_roles = [beatles_owner, beatles_member, monsters_owner, monsters_member]
    for org_role in org_roles:
        session.add(org_role)
    vocalists_member = TeamRole(name="MEMBER", team=vocalists, users=[paul])
    vocalists_maintainer = TeamRole(name="MAINTAINER", team=vocalists, users=[john])
    percussion_member = TeamRole(name="MEMBER", team=percussion, users=[])
    percussion_maintainer = TeamRole(name="MAINTAINER", team=percussion, users=[ringo])
    scarers_member = TeamRole(name="MEMBER", team=scarers, users=[randall])
    scarers_maintainer = TeamRole(name="MAINTAINER", team=scarers, users=[sully])
    team_roles = [
        vocalists_member,
        vocalists_maintainer,
        percussion_member,
        percussion_maintainer,
        scarers_member,
        scarers_maintainer,
    ]
    for team_role in team_roles:
        session.add(team_role)

    session.commit()


@pytest.fixture
def test_db_session():
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)

    Session = sessionmaker(bind=engine)
    session = Session()

    load_fixture_data(session)

    return session


def test_get_user_resources_and_roles(test_db_session):
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    resource_roles = oso_roles.get_user_resources_and_roles(
        test_db_session, john, Organization
    )
    assert len(resource_roles) == 1
    assert resource_roles[0][0].name == "The Beatles"
    assert resource_roles[0][1].name == "OWNER"


def test_get_user_roles_for_resource(test_db_session):
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    beatles = test_db_session.query(Organization).filter_by(name="The Beatles").first()
    resource_roles = oso_roles.get_user_roles_for_resource(
        test_db_session, john, beatles
    )
    assert len(resource_roles) == 1
    assert resource_roles[0].name == "OWNER"


def test_get_resource_users_and_roles(test_db_session):
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()
    users = oso_roles.get_resource_users_and_roles(test_db_session, abbey_road)
    assert len(users)
    assert users[0][0].email == "john@beatles.com"
    assert users[0][1].name == "READ"
    assert users[1][0].email == "paul@beatles.com"
    assert users[0][1].name == "READ"


def test_get_resource_users_with_role(test_db_session):
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()
    users = oso_roles.get_resource_users_with_role(test_db_session, abbey_road, "READ")
    assert len(users) == 2
    assert users[0].email == "john@beatles.com"
    assert users[1].email == "paul@beatles.com"


def test_add_user_role(test_db_session):
    ringo = test_db_session.query(User).filter_by(email="ringo@beatles.com").first()
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()

    roles = oso_roles.get_user_roles_for_resource(test_db_session, ringo, abbey_road)
    assert len(roles) == 0

    oso_roles.add_user_role(test_db_session, ringo, abbey_road, "READ")

    roles = oso_roles.get_user_roles_for_resource(test_db_session, ringo, abbey_road)
    assert len(roles) == 1
    assert roles[0].name == "READ"

    with pytest.raises(Exception):
        oso_roles.add_user_role(test_db_session, ringo, abbey_road, "NOT_A_REAL_ROLE")


def test_delete_user_role(test_db_session):
    # Test with explicit role arg
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()

    roles = oso_roles.get_user_roles_for_resource(test_db_session, john, abbey_road)
    assert len(roles) == 1

    oso_roles.delete_user_role(test_db_session, john, abbey_road, "READ")

    roles = oso_roles.get_user_roles_for_resource(test_db_session, john, abbey_road)
    assert len(roles) == 0

    # Test without explicit role arg
    paul = test_db_session.query(User).filter_by(email="paul@beatles.com").first()
    roles = oso_roles.get_user_roles_for_resource(test_db_session, paul, abbey_road)
    assert len(roles) == 1

    oso_roles.delete_user_role(test_db_session, paul, abbey_road)

    roles = oso_roles.get_user_roles_for_resource(test_db_session, paul, abbey_road)
    assert len(roles) == 0

    # Test trying to delete non-existent role raises exception
    with pytest.raises(Exception):
        oso_roles.delete_user_role(test_db_session, paul, abbey_road, "READ")


def test_reassign_user_role(test_db_session):
    john = test_db_session.query(User).filter_by(email="john@beatles.com").first()
    abbey_road = test_db_session.query(Repository).filter_by(name="Abbey Road").first()

    roles = oso_roles.get_user_roles_for_resource(test_db_session, john, abbey_road)
    assert len(roles) == 1
    assert roles[0].name == "READ"

    oso_roles.reassign_user_role(test_db_session, john, abbey_road, "WRITE")

    roles = oso_roles.get_user_roles_for_resource(test_db_session, john, abbey_road)
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
