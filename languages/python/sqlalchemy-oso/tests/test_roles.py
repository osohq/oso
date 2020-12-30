# type: ignore

import pytest
import datetime

from sqlalchemy import create_engine
from sqlalchemy.types import Integer, String, DateTime
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso import roles as oso_roles, register_models
from sqlalchemy_oso.roles import enable_roles
from sqlalchemy_oso.session import set_get_session
from oso import Oso, Variable

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
    Base,
    User,
    Repository,
    ["READ", "TRIAGE", "WRITE", "MAINTAIN", "ADMIN"],
)


class RepositoryRole(Base, RepositoryRoleMixin):
    def repr(self):
        return {"id": self.id, "name": str(self.name)}


# For the tests, make OrganizationRoles NOT mutually exclusive
OrganizationRoleMixin = oso_roles.resource_role_class(
    Base, User, Organization, ["OWNER", "MEMBER", "BILLING"], mutually_exclusive=False
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


# TEST FIXTURES


@pytest.fixture
def test_db_session():
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)

    Session = sessionmaker(bind=engine)
    session = Session()

    load_fixture_data(session)

    return session


@pytest.fixture
def oso_with_session(test_db_session):
    oso = Oso()
    set_get_session(oso, lambda: test_db_session)
    register_models(oso, Base)

    return oso


@pytest.fixture
def john(test_db_session):
    return test_db_session.query(User).filter_by(email="john@beatles.com").first()


@pytest.fixture
def paul(test_db_session):
    return test_db_session.query(User).filter_by(email="paul@beatles.com").first()


@pytest.fixture
def ringo(test_db_session):
    return test_db_session.query(User).filter_by(email="ringo@beatles.com").first()


@pytest.fixture
def abbey_road(test_db_session):
    return test_db_session.query(Repository).filter_by(name="Abbey Road").first()


@pytest.fixture
def beatles(test_db_session):
    return test_db_session.query(Organization).filter_by(name="The Beatles").first()


def test_user_resources_relationship_fields(test_db_session):
    beatles = test_db_session.query(Organization).filter_by(name="The Beatles").first()
    users = beatles.users
    users.sort(key=lambda x: x.email)
    assert len(users) == 3
    assert users[0].email == "john@beatles.com"


def test_resource_users_relationship_fields(john):
    orgs = john.organizations
    assert len(orgs) == 1
    assert orgs[0].name == "The Beatles"


def test_get_user_resources_and_roles(test_db_session, john):
    # Test with ORM method
    roles = john.organization_roles
    assert len(roles) == 1
    assert roles[0].name == "OWNER"
    assert roles[0].organization.name == "The Beatles"

    # Test with oso method
    roles = oso_roles.get_user_roles(test_db_session, john, Organization)
    assert len(roles) == 1
    assert roles[0].name == "OWNER"
    assert roles[0].organization.name == "The Beatles"


def test_get_user_roles_for_resource(test_db_session, john, beatles):
    # Test with ORM method
    resource_roles = (
        test_db_session.query(OrganizationRole)
        .filter_by(user=john, organization=beatles)
        .all()
    )
    assert len(resource_roles) == 1
    assert resource_roles[0].name == "OWNER"

    # Test with oso method
    resource_roles = oso_roles.get_user_roles(
        test_db_session, john, Organization, beatles.id
    )
    assert len(resource_roles) == 1
    assert resource_roles[0].name == "OWNER"


def test_get_resource_roles(test_db_session, abbey_road):
    # Test with ORM method
    user_roles = abbey_road.roles
    assert user_roles[0].user.email == "john@beatles.com"
    assert user_roles[0].name == "READ"
    assert user_roles[1].user.email == "paul@beatles.com"
    assert user_roles[0].name == "READ"

    # Test with oso method
    user_roles = oso_roles.get_resource_roles(test_db_session, abbey_road)
    assert user_roles[0].user.email == "john@beatles.com"
    assert user_roles[0].name == "READ"
    assert user_roles[1].user.email == "paul@beatles.com"
    assert user_roles[0].name == "READ"


def test_get_resource_users_by_role(test_db_session, abbey_road):
    # Test with ORM method
    users = (
        test_db_session.query(User)
        .join(RepositoryRole)
        .filter_by(repository=abbey_road, name="READ")
        .all()
    )
    assert len(users) == 2
    assert users[0].email == "john@beatles.com"
    assert users[1].email == "paul@beatles.com"

    # Test with oso method
    users = oso_roles.get_resource_users_by_role(test_db_session, abbey_road, "READ")
    assert len(users) == 2
    assert users[0].email == "john@beatles.com"
    assert users[1].email == "paul@beatles.com"


def test_add_user_role(test_db_session, abbey_road, ringo, beatles):
    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=ringo, repository=abbey_road)
        .all()
    )
    assert len(roles) == 0

    # Test can't add invalid role
    with pytest.raises(ValueError):
        oso_roles.add_user_role(test_db_session, ringo, abbey_road, "FAKE", commit=True)

    # Test adding valid role
    oso_roles.add_user_role(test_db_session, ringo, abbey_road, "READ", commit=True)

    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=ringo, repository=abbey_road)
        .all()
    )
    assert len(roles) == 1
    assert roles[0].name == "READ"

    # ensure user cannot have duplicate role
    with pytest.raises(Exception):
        oso_roles.add_user_role(test_db_session, ringo, abbey_road, "READ", commit=True)

    # ensure user cannot have two roles for the same resource if `mutually_exclusive=True`
    with pytest.raises(Exception):
        oso_roles.add_user_role(
            test_db_session, ringo, abbey_road, "WRITE", commit=True
        )

    roles = (
        test_db_session.query(OrganizationRole)
        .filter_by(user=ringo, organization=beatles)
        .order_by(OrganizationRole.name)
        .all()
    )
    assert len(roles) == 1
    assert roles[0].name == "MEMBER"

    # ensure user cannot have two roles for the same resource
    with pytest.raises(Exception):
        oso_roles.add_user_role(test_db_session, ringo, beatles, "MEMBER", commit=True)

    # ensure user can have two roles for the same resource if `mutually_exclusive=False`
    oso_roles.add_user_role(test_db_session, ringo, beatles, "BILLING")

    roles = (
        test_db_session.query(OrganizationRole)
        .filter_by(user=ringo, organization=beatles)
        .order_by(OrganizationRole.name)
        .all()
    )
    assert len(roles) == 2
    assert roles[0].name == "BILLING"


def test_delete_user_role(test_db_session, john, paul, abbey_road):
    # Test with explicit role arg
    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=john, repository=abbey_road, name="READ")
        .all()
    )
    assert len(roles) == 1

    oso_roles.delete_user_role(test_db_session, john, abbey_road, "READ")

    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=john, repository=abbey_road, name="READ")
        .all()
    )
    assert len(roles) == 0

    # Test with no role arg
    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=paul, repository=abbey_road)
        .all()
    )
    assert len(roles) == 1

    oso_roles.delete_user_role(test_db_session, paul, abbey_road)

    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=paul, repository=abbey_road)
        .all()
    )
    assert len(roles) == 0


def test_reassign_user_role(test_db_session, john, abbey_road):
    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=john, repository=abbey_road)
        .all()
    )
    assert len(roles) == 1
    assert roles[0].name == "READ"

    oso_roles.reassign_user_role(test_db_session, john, abbey_road, "WRITE")

    roles = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=john, repository=abbey_road)
        .all()
    )
    assert len(roles) == 1
    assert roles[0].name == "WRITE"


def test_set_get_session(oso_with_session):
    test_str = """get_repo(name: String) if
                    session = OsoSession.get() and
                    repo = session.query(Repository).filter_by(name: name).first() and
                    repo.name = name;
                    """

    oso = oso_with_session

    oso.load_str(test_str)
    results = oso.query_rule("get_repo", "Abbey Road")
    assert next(results)
    results = oso.query_rule("get_repo", "Abbey Road")
    assert next(results)


def test_duplicate_resource_role():
    with pytest.raises(ValueError):
        oso_roles.resource_role_class(
            Base,
            User,
            Repository,
            ["READ", "TRIAGE", "WRITE", "MAINTAIN", "ADMIN"],
        )


def test_enable_roles(
    test_db_session, oso_with_session, john, ringo, abbey_road, beatles
):
    oso = oso_with_session
    enable_roles(oso)

    # Get test data
    read_repo_role = (
        test_db_session.query(RepositoryRole)
        .filter_by(user=john, repository=abbey_road)
        .first()
    )
    org_owner_role = (
        test_db_session.query(OrganizationRole)
        .filter_by(user=john, organization=beatles)
        .first()
    )

    # test base `resource_role_applies_to`
    results = list(
        oso.query_rule(
            "resource_role_applies_to", abbey_road, Variable("role_resource")
        )
    )
    assert len(results) == 1
    assert results[0].get("bindings").get("role_resource") == abbey_road

    # test custom `resource_role_applies_to` rules (for nested resources)
    resource_role_applies_to_str = """resource_role_applies_to(repo: Repository, parent_org) if
        parent_org := repo.organization and
        parent_org matches Organization;
        """
    oso.load_str(resource_role_applies_to_str)
    results = list(
        oso.query_rule(
            "resource_role_applies_to", abbey_road, Variable("role_resource")
        )
    )
    results.sort(key=lambda x: x.get("bindings").get("role_resource").name)
    assert len(results) == 2
    assert results[0].get("bindings").get("role_resource") == abbey_road
    assert results[1].get("bindings").get("role_resource") == beatles

    # test `user_in_role` for RepositoryRole
    results = list(oso.query_rule("user_in_role", john, Variable("role"), abbey_road))
    assert len(results) == 1
    assert results[0].get("bindings").get("role").name == "READ"

    # test `user_in_role` for OrganizationRole
    results = list(oso.query_rule("user_in_role", john, Variable("role"), beatles))
    assert len(results) == 1
    assert results[0].get("bindings").get("role").name == "OWNER"

    # test `inherits_role` and `resource_role_order`
    # make sure `inherits_role` returns nothing without a role order rule
    results = list(
        oso.query_rule("inherits_role", org_owner_role, Variable("inherited_role"))
    )
    assert len(results) == 0

    # test role_order rule
    role_order_str = 'organization_role_order(["OWNER", "MEMBER", "BILLING"]);'
    oso.load_str(role_order_str)

    results = list(
        oso.query_rule("inherits_role", org_owner_role, Variable("inherited_role"))
    )
    results.sort(key=lambda x: x.get("bindings").get("inherited_role").name)
    assert len(results) == 2
    assert results[0].get("bindings").get("inherited_role").name == "BILLING"
    assert results[1].get("bindings").get("inherited_role").name == "MEMBER"

    # make sure this query fails before any rules are added
    results = list(oso.query_rule("role_allow", john, "READ", abbey_road))
    assert len(results) == 0

    # test basic `role_allow` rule
    role_allow_str = (
        'role_allow(role: RepositoryRole{name: "READ"}, "READ", repo: Repository);'
    )

    oso.load_str(role_allow_str)
    results = list(oso.query_rule("role_allow", read_repo_role, "READ", abbey_road))
    assert len(results) == 1

    # test `role_allow` rule using nested resource
    nested_role_allow_str = (
        'role_allow(role: OrganizationRole{name: "MEMBER"}, "READ", repo: Repository);'
    )
    oso.load_str(nested_role_allow_str)
    results = list(oso.query_rule("role_allow", org_owner_role, "READ", abbey_road))
    assert len(results) == 1

    # test top-level `allow`
    results = list(oso.query_rule("allow", john, "READ", abbey_road))
    assert len(results) == 2

    results = list(oso.query_rule("allow", ringo, "READ", abbey_road))
    assert len(results) == 1
