# Roles 2 tests
import pytest

from sqlalchemy import create_engine
from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso import register_models, authorized_sessionmaker
from sqlalchemy_oso.roles2 import OsoRoles

from oso import Oso


Base = declarative_base(name="RoleBase")


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True)
    name = Column(String())


class Organization(Base):
    __tablename__ = "organizations"

    id = Column(String(), primary_key=True)


class Repository(Base):
    __tablename__ = "repositories"

    id = Column(String(), primary_key=True)
    org_id = Column(String(), ForeignKey("organizations.id"))
    org = relationship("Organization")


class Issue(Base):
    __tablename__ = "issues"

    id = Column(String(), primary_key=True)
    repo_id = Column(String(), ForeignKey("repositories.id"))
    repo = relationship("Repository")


@pytest.fixture
def engine():
    engine = create_engine("sqlite:///:memory:")
    return engine


@pytest.fixture
def init_oso(engine):

    # Initialize Oso and OsoRoles
    # ---------------------------
    Session = sessionmaker(bind=engine)
    session = Session()

    oso = Oso()
    register_models(oso, Base)

    roles = OsoRoles(oso, Base, User, Session)
    roles.enable()

    # @NOTE: Right now this has to happen after enabling oso roles to get the
    #        tables.
    Base.metadata.create_all(engine)

    return (oso, roles, session)


@pytest.fixture
def authorized_sessionmaker(init_oso, engine):
    oso, oso_roles, _ = init_oso
    oso.actor = None
    oso.action = None

    AuthSessionmaker = authorized_sessionmaker(
        bind=engine,
        get_oso=lambda: oso,
        get_user=lambda: oso.actor,
        get_action=lambda: oso.action,
    )

    return AuthSessionmaker


## TEST RESOURCE CONFIGURATION
# test cases
# - duplicate permission
# - assign permission that wasn't declared
# - imply role that wasn't declared
# - imply role without valid relationship
# - assign permission without valid relationship
# - use resource predicate with incorrect arity
# - use resource predicate without defining actions/roles
# - use resource predicate with field types


def test_resource_actions(init_oso):

    # init
    oso, oso_roles, session = init_oso
    # - test with only actions
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ];
    """
    oso.load_str(policy)
    oso_roles.configure()


def test_duplicate_action(init_oso):
    # - duplicate action
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "invite"
        ];
    """
    oso.load_str(policy)

    with pytest.raises(Exception):
        oso_roles.configure()


def test_undeclared_permission(init_oso):
    # - assign permission that wasn't declared
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            org_member: {
                perms: ["create_repo"]
            }
        };
    """
    oso.load_str(policy)

    with pytest.raises(Exception) as e:
        oso_roles.configure()

    # TODO: Make this an actual error, not an assert
    assert e.typename != "AssertionError"


def test_undeclared_role(init_oso):
    # - imply role that wasn't declared
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            org_member: {
                implies: ["fake_role"]
            }
        };
    """
    oso.load_str(policy)
    with pytest.raises(Exception):
        oso_roles.configure()


def test_invalid_role_implication(init_oso):
    # - imply role without valid relationship
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            org_member: {
                implies: ["repo_read"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            repo_read: {
                perms: ["pull"]
            }
        };
    """
    oso.load_str(policy)
    with pytest.raises(Exception):
        oso_roles.configure()


def test_invalid_role_permission(init_oso):
    # - assign permission without valid relationship
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            org_member: {
                perms: ["repo:push"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ];
    """
    oso.load_str(policy)
    with pytest.raises(Exception):
        oso_roles.configure()


def test_incorrect_arity_resource(init_oso):
    # - use resource predicate with incorrect arity
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions) if
        actions = [
            "invite"
        ];
    """
    oso.load_str(policy)
    with pytest.raises(Exception):
        oso_roles.configure()


def test_undefined_resource_arguments(init_oso):
    # - use resource predicate without defining actions/roles
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles);
    """
    oso.load_str(policy)
    with pytest.raises(Exception):
        oso_roles.configure()


def test_wrong_type_resource_arguments(init_oso):
    # - use resource predicate with field types
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                # incorrect key name
                actions: ["invite"]
            }
        };
    """
    oso.load_str(policy)
    with pytest.raises(Exception) as e:
        oso_roles.configure()


def test_roles(init_oso, authorized_sessionmaker):
    oso, oso_roles, session = init_oso

    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            org_member: {
                perms: ["create_repo"],
                implies: ["repo_read"]
            },
            org_owner: {
                perms: ["invite"],
                implies: ["org_member", "repo_write"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            repo_write: {
                perms: ["push", "issue:edit"],
                implies: ["repo_read"]
            },
            repo_read: {
                perms: ["pull"]
            }
        };

    resource(_type: Issue, "issue", actions, _) if
        actions = [
            "edit"
        ];

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    parent(issue: Issue, parent_repo: Repository) if
        issue.repo = parent_repo;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    # tbd on the name for this, but this is what used to happy lazily.
    # it reads the config from the policy and sets everything up.
    oso_roles.configure()

    # Create sample data
    # -------------------
    apple = Organization(id="apple")
    osohq = Organization(id="osohq")

    ios = Repository(id="ios", org=apple)
    oso_repo = Repository(id="oso", org=osohq)
    demo_repo = Repository(id="demo", org=osohq)

    laggy = Issue(id="laggy", repo=ios)
    bug = Issue(id="bug", repo=oso_repo)

    leina = User(name="leina")
    steve = User(name="steve")

    objs = [leina, steve, apple, osohq, ios, oso_repo, demo_repo, laggy, bug]
    for obj in objs:
        session.add(obj)
    session.commit()

    # @NOTE: Need the users and resources in the db before assigning roles
    # so you have to call session.commit() first.
    oso_roles.assign_role(leina, osohq, "org_owner", session=session)
    oso_roles.assign_role(steve, osohq, "org_member", session=session)

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "create_repo", osohq)
    assert oso.is_allowed(leina, "push", oso_repo)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "edit", bug)

    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "create_repo", osohq)
    assert not oso.is_allowed(steve, "push", oso_repo)
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert not oso.is_allowed(steve, "edit", bug)

    assert not oso.is_allowed(leina, "edit", laggy)
    assert not oso.is_allowed(steve, "edit", laggy)

    oso.actor = leina
    oso.action = "pull"
    auth_session = authorized_sessionmaker()

    results = auth_session.query(Repository).all()
    assert len(results) == 2
    result_ids = [repo.id for repo in results]
    assert oso_repo.id in result_ids
    assert demo_repo.id in result_ids
    assert ios.id not in result_ids
