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
def auth_sessionmaker(init_oso, engine):
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


# TEST OsoRoles Initialization
# - Passing an auth session to OsoRoles raises an exception
# - Passing a session instead of Session factory to OsoRoles raises an exception
# - Passing a non-SQLAlchemy user model to OsoRoles raises an exception
# - Passing a bad declarative_base to OsoRoles raises an exception


def test_oso_roles_init(auth_sessionmaker):
    oso = Oso()
    register_models(oso, Base)

    # - Passing an auth session to OsoRoles raises an exception
    with pytest.raises(Exception):
        roles = OsoRoles(oso, Base, User, auth_sessionmaker)

    Session = sessionmaker(bind=engine)
    session = Session()

    # - Passing a session instead of Session factory to OsoRoles raises an exception
    with pytest.raises(Exception):
        OsoRoles(oso, Base, User, session)

    class FakeClass:
        pass

    # - Passing a non-SQLAlchemy user model to OsoRoles raises an exception
    with pytest.raises(Exception):
        roles = OsoRoles(oso, Base, FakeClass, Session)

    # - Passing a bad declarative_base to OsoRoles raises an exception
    with pytest.raises(Exception):
        roles = OsoRoles(oso, FakeClass, User, Session)


# TEST RESOURCE CONFIGURATION
# Role declaration:
# - [ ] duplicate role name throws an error


# Role-permission assignment:
# - [ ] duplicate permission throws an error
# - [ ] assigning permission that wasn't declared throws an error
# - [ ] assigning permission without valid relationship throws an error
# - [ ] assigning permission on related role type errors if role exists for permission resource
# - [ ] assigning the same permission to two roles where one implies the other throws an error

# Role implications:
# - [ ] implying role that wasn't declared throws an error
# - [ ] implying role without valid relationship throws an error

# Resource predicate:
# - [x] only define actions, not roles
# - [ ] using resource predicate with incorrect arity throws an error
# - [ ] using resource predicate without defining actions/roles throws an error
# - [ ] using resource predicate with field types throws an error


def test_duplicate_role_name(init_oso):
    # duplicate role name throws an error
    # Organization and Repository resources both have role named "member"
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                perms: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            member: {
                perms: ["pull"]
            }
        };
    """
    oso.load_str(policy)

    with pytest.raises(Exception):
        oso_roles.configure()


def test_resource_actions(init_oso):
    # only define actions, not roles
    oso, oso_roles, session = init_oso
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


def test_role_implication_without_relationship(init_oso):
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


def test_role_permission_without_relationship(init_oso):
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


def test_invalid_role_permission(init_oso):
    # assigning permission on related role type errors if role exists for permission resource
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            org_member: {
                # THIS IS NOT ALLOWED
                perms: ["repo:push"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            repo_read: {
                perms: ["push"]
            }

        };

    parent(repo: Repository, parent_org: Organization) if
        repository.org = parent_org;
    """

    oso.load_str(policy)
    # TODO: make this not an AssertionError
    with pytest.raises(Exception):
        oso_roles.configure()


def test_permission_assignment_to_implied_role(init_oso):
    # assigning the same permission to two roles where one implies the other throws an error
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            org_member: {
                perms: ["invite"]
            },
            org_owner: {
                perms: ["invite"],
                implies: ["org_member"]
            }

        };
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
    with pytest.raises(Exception):
        oso_roles.configure()


# TEST CHECK API
# Homogeneous role-permission assignment:
# - Adding a permission of same resource type to a role grants assignee access
# - Modifying a permission of same resource type on a role modifies assignee access
# - Removing a permission of same resource type from a role revokes assignee access

# Heterogeneous role-permission assignment:
# - Adding a permission of related resource type to a role grants assignee access
# - Modifying a permission of related resource type on a role modifies assignee access
# - Removing a permission of related resource type from a role revokes assignee access

# Homogeneous role implications:
# - Adding a role implication of same resource type to a role grants assignee access
# - Modifying a role implication of same resource type to a role modifies assignee access
# - Removing a role implication of same resource type from a role revokes assignee access

# Heterogeneous role implications:
# - Adding a role implication of related resource type to a role grants assignee access
# - Modifying a role implication of related resource type to a role modifies assignee access
# - Removing a role implication of related resource type from a role revokes assignee access

# TEST WRITE API
# User-role assignment:
# - Implied roles are mutually exclusive on user-role assignment


# Homogeneous role-permission assignment:
def test_add_homogenous_role_perm():
    # - Adding a permission of same resource type to a role grants assignee access
    pass


def test_remove_homogenous_role_perm():
    # - Removing a permission of same resource type from a role revokes assignee access
    pass


def test_modify_homogenous_role_perm():
    # - Modifying a permission of same resource type on a role modifies assignee access
    pass


def test_roles(init_oso, auth_sessionmaker):
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
    auth_session = auth_sessionmaker()

    results = auth_session.query(Repository).all()
    assert len(results) == 2
    result_ids = [repo.id for repo in results]
    assert oso_repo.id in result_ids
    assert demo_repo.id in result_ids
    assert ios.id not in result_ids
