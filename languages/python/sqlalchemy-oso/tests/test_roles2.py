# Roles 2 tests
import pytest
import psycopg2
import random
import string
import os

from sqlalchemy import create_engine
from sqlalchemy.pool import NullPool
from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker, close_all_sessions
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso import authorized_sessionmaker, SQLAlchemyOso

from oso import OsoError

pg_host = os.environ.get("POSTGRES_HOST")
pg_port = os.environ.get("POSTGRES_PORT")
pg_user = os.environ.get("POSTGRES_USER")
pg_pass = os.environ.get("POSTGRES_PASSWORD")

databases = ["sqlite"]
if pg_host is not None:
    databases.append("postgres")


@pytest.fixture(params=databases)
def engine(request):
    if request.param == "postgres":
        # Create a new database to run the tests.
        id = "".join(random.choice(string.ascii_lowercase) for i in range(10))
        name = f"roles_test_{id}"

        connect_string = "postgresql://"
        kwargs = {"host": pg_host}
        if pg_user is not None:
            kwargs["user"] = pg_user
            connect_string += pg_user
        if pg_pass is not None:
            kwargs["password"] = pg_pass
            connect_string += ":" + pg_user
        connect_string += "@" + pg_host
        if pg_port is not None:
            kwargs["port"] = pg_port
            connect_string += ":" + pg_port
        conn = psycopg2.connect(**kwargs)
        conn.autocommit = True
        cursor = conn.cursor()
        cursor.execute(f"create database {name}")
        conn.close()

        # Run tests.
        engine = create_engine(f"{connect_string}/{name}", poolclass=NullPool)
        yield engine
        engine.dispose()
        close_all_sessions()

        # Destroy database.
        conn = psycopg2.connect(**kwargs)
        conn.autocommit = True
        cursor = conn.cursor()
        cursor.execute(f"drop database if exists {name}")
        conn.close()
    elif request.param == "sqlite":
        engine = create_engine("sqlite:///:memory:")
        yield engine


@pytest.fixture
def Base():
    base = declarative_base(name="RoleBase")

    return base


@pytest.fixture
def User(Base):
    class User(Base):
        __tablename__ = "users"

        id = Column(Integer, primary_key=True)
        name = Column(String())

    return User


@pytest.fixture
def Organization(Base):
    class Organization(Base):
        __tablename__ = "organizations"

        id = Column(String(), primary_key=True)

    return Organization


@pytest.fixture
def Repository(Base):
    class Repository(Base):
        __tablename__ = "repositories"

        id = Column(String(), primary_key=True)
        org_id = Column(String(), ForeignKey("organizations.id"), index=True)
        org = relationship("Organization")

    return Repository


@pytest.fixture
def Issue(Base):
    class Issue(Base):
        __tablename__ = "issues"

        id = Column(String(), primary_key=True)
        repo_id = Column(String(), ForeignKey("repositories.id"))
        repo = relationship("Repository")

    return Issue


@pytest.fixture
def init_oso(engine, Base, User, Organization, Repository, Issue):
    # Initialize Oso and OsoRoles
    # ---------------------------
    Session = sessionmaker(bind=engine)
    session = Session()

    oso = SQLAlchemyOso(Base)
    oso.enable_roles(User, Session)

    # @NOTE: Right now this has to happen after enabling oso roles to get the
    #        tables.
    Base.metadata.create_all(engine)

    return (oso, session)


@pytest.fixture
def auth_sessionmaker(init_oso, engine):
    oso, _ = init_oso
    oso.actor = None
    oso.checked_permissions = None

    AuthSessionmaker = authorized_sessionmaker(
        bind=engine,
        get_oso=lambda: oso,
        get_user=lambda: oso.actor,
        get_checked_permissions=lambda: oso.checked_permissions,
    )

    return AuthSessionmaker


@pytest.fixture
def sample_data(init_oso, Organization, Repository, User, Issue):
    _, session = init_oso
    # Create sample data
    # -------------------
    apple = Organization(id="apple")
    osohq = Organization(id="osohq")

    ios = Repository(id="ios", org=apple)
    oso_repo = Repository(id="oso", org=osohq)
    demo_repo = Repository(id="demo", org=osohq)

    ios_laggy = Issue(id="laggy", repo=ios)
    oso_bug = Issue(id="bug", repo=oso_repo)

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


# TEST OsoRoles Initialization
# - Passing an auth session to OsoRoles raises an exception
# - Passing a session instead of Session factory to OsoRoles raises an exception
# - Passing a non-SQLAlchemy user model to OsoRoles raises an exception
# - Passing a bad declarative_base to OsoRoles raises an exception


def test_oso_roles_init(engine, auth_sessionmaker, Base, User):
    oso = SQLAlchemyOso(Base)

    # - Passing an auth session to OsoRoles raises an exception
    with pytest.raises(OsoError):
        oso.enable_roles(
            user_model=User,
            session_maker=auth_sessionmaker,
        )

    Session = sessionmaker(bind=engine)
    session = Session()

    # - Passing a session instead of Session factory to OsoRoles raises an exception
    with pytest.raises(AttributeError):
        oso.enable_roles(User, session)

    class FakeClass:
        pass

    # - Passing a non-SQLAlchemy user model to OsoRoles raises an exception
    with pytest.raises(TypeError):
        oso.enable_roles(FakeClass, Session)

    # - Passing a bad declarative_base to OsoRoles raises an exception
    with pytest.raises(AttributeError):
        SQLAlchemyOso(FakeClass)

    # - Calling a roles-specific method before calling `enable_roles` fails
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


# TEST RESOURCE CONFIGURATION
# Role declaration:
# - [x] duplicate role name throws an error
# - [x] defining role with no permissions/implications throws an error

# Role-permission assignment:
# - [x] duplicate permission throws an error
# - [x] assigning permission that wasn't declared throws an error
# - [x] assigning permission with bad namespace throws an error
# - [x] assigning permission without valid relationship throws an error
# - [x] assigning permission on related role type errors if role exists for permission resource
# - [x] assigning the same permission to two roles where one implies the other throws an error

# Role implications:
# - [x] implying role that wasn't declared throws an error
# - [x] implying role without valid relationship throws an error

# Resource predicate:
# - [x] only define roles, no actions (role has actions/implications from different resource)
# - [x] only define actions, not roles
# - [x] using resource predicate with incorrect arity throws an error
# - [x] using resource predicate without defining actions/roles throws an error
# - [x] using resource predicate with field types throws an error
# - [x] duplicate resource name throws an error

# Role allows:
# - [ ] calling `roles.configure()` without calling `Roles.role_allows()` from policy issues warning
#   TODO write test

# Relationships:
# - [x] multiple dot lookups throws an error for now
# - [x] nonexistent attribute lookup throws an error for now
# - [x] relationships without resource definition throws an error


def test_empty_role(init_oso):
    # defining role with no permissions/implications throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {}
        };
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()
    pass


def test_bad_namespace_perm(init_oso):
    # - assigning permission with bad namespace throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                permissions: ["repo:pull"]
            }
        };
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


# TODO
def test_resource_with_roles_no_actions(init_oso, sample_data):
    # - only define roles, no actions (role has actions/implications from different resource)
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", _, roles) if
        roles = {
            member: {
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

        parent(repo: Repository, parent_org: Organization) if
            repo.org = parent_org;

        allow(actor, action, resource) if
            Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    oso.roles.synchronize_data()

    leina = sample_data["leina"]
    steve = sample_data["steve"]
    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]

    oso.roles.assign_role(leina, osohq, "member", session)
    oso.roles.assign_role(steve, oso_repo, "reader", session)

    session.commit()

    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(steve, "pull", oso_repo)


def test_duplicate_resource_name(init_oso):
    # - duplicate resource name throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    # DUPLICATE RESOURCE NAME "org"
    resource(_type: Repository, "org", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_nested_dot_relationship(init_oso):
    # - multiple dot lookups throws an error for now
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ];

    parent(issue, parent_org) if
        issue.repo.org = parent_org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_bad_relationship_lookup(init_oso):
    # - nonexistent attribute lookup throws an error for now
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, _) if
        actions = [
            "pull"
        ];

    parent(repo: Repository, parent_org: Organization) if
        # INCORRECT FIELD NAME
        repo.organization = parent_org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_relationship_without_specializer(init_oso):
    oso, session = init_oso
    policy = """
    resource(_type: Repository, "repo", actions, _) if
        actions = [
            "pull"
        ];

    parent(repo, parent_org: Organization) if
        repo.org = parent_org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_relationship_without_resources(init_oso):
    oso, session = init_oso
    policy = """
    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_duplicate_role_name_same_resource(init_oso):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite", "create_repo"
        ] and
        roles = {
            owner: {
                permissions: ["invite"],
                implies: ["member", "repo:member"]
            },
            owner: {
                permissions: ["create_repo"]
            }
        };
        """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_duplicate_role_name_different_resources(init_oso, sample_data):
    # duplicate role name throws an error
    # Organization and Repository resources both have role named "member"
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite", "create_repo"
        ] and
        roles = {
            owner: {
                permissions: ["invite"],
                implies: ["member", "repo:member"]
            },
            member: {
                permissions: ["create_repo"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            member: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]
    gabe = sample_data["gabe"]

    oso.roles.assign_role(leina, osohq, "owner", session)
    oso.roles.assign_role(steve, oso_repo, "member", session)
    oso.roles.assign_role(gabe, osohq, "member", session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "pull", oso_repo)

    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "pull", oso_repo)

    assert not oso.is_allowed(gabe, "invite", osohq)
    assert oso.is_allowed(gabe, "create_repo", osohq)
    assert not oso.is_allowed(gabe, "pull", oso_repo)


def test_resource_actions(init_oso):
    # only define actions, not roles
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ];
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()


def test_duplicate_action(init_oso):
    # - duplicate action
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "invite"
        ];
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_undeclared_permission(init_oso):
    # - assign permission that wasn't declared
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                permissions: ["create_repo"]
            }
        };
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_undeclared_role(init_oso):
    # - imply role that wasn't declared
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                implies: ["fake_role"]
            }
        };
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_role_implication_without_relationship(init_oso):
    # - imply role without valid relationship
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_role_permission_without_relationship(init_oso):
    # - assign permission without valid relationship
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                permissions: ["repo:push"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ];
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_invalid_role_permission(init_oso):
    # assigning permission on related role type errors if role exists for permission resource
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                # THIS IS NOT ALLOWED
                permissions: ["repo:push"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["push"]
            }

        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;
    """

    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_permission_assignment_to_implied_role(init_oso):
    # assigning the same permission to two roles where one implies the other throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                permissions: ["invite"]
            },
            owner: {
                permissions: ["invite"],
                implies: ["org:member"]
            }

        };
    """

    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_incorrect_arity_resource(init_oso):
    # - use resource predicate with incorrect arity
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions) if
        actions = [
            "invite"
        ];
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_undefined_resource_arguments(init_oso):
    # - use resource predicate without defining actions/roles
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles);
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


def test_wrong_type_resource_arguments(init_oso):
    # - use resource predicate with field types
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                # incorrect key name
                actions: ["invite"]
            }
        };
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.roles.synchronize_data()


# TEST CHECK API
# Homogeneous role-permission assignment:
# - [x] Adding a permission of same resource type to a role grants assignee access
# - [x] Modifying a permission of same resource type on a role modifies assignee access
# - [x] Removing a permission of same resource type from a role revokes assignee access

# Parent->child role-permission assignment:
# - [x] Adding a permission of child resource type to a role grants assignee access
# - [x] Removing a permission of child resource type from a role revokes assignee access

# Grandparent->child role-permission assignment:
# - [x] Adding a permission of grandchild resource type to a role grants assignee access
# - [x] Removing a permission of grandchild resource type from a role revokes assignee access

# Homogeneous role implications:
# - [x] Adding a role implication of same resource type to a role grants assignee access
# - [x] Removing a role implication of same resource type from a role revokes assignee access

# Parent->child role implications:
# - [x] Adding a role implication of child resource type to a role grants assignee access to child
# - [x] Removing a role implication of child resource type from a role revokes assignee access to child

# Grandparent->child role implications:
# - [x] Adding a role implication of grandchild resource type to a role grants assignee access to grandchild
#       without intermediate parent resource

# Chained role implications:
# - [x] Adding a role implication from grandparent->parent->child resource role types grants assignee of grandparent role
#   access to grandchild resource

# Overlapping role assignments:
# - [x] Assigning a more permissive and less permissive role to the same user grants most permissive access

# Overlapping role assignments:
def test_overlapping_permissions(init_oso, sample_data):
    # - Assigning a more permissive and less permissive role to the same user grants most permissive access
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"]
            },
            writer: {
                # writer is more permissive than reader
                permissions: ["push"],
                implies: ["reader"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # writer is more permissive than member
    oso.roles.assign_role(leina, osohq, "member")
    oso.roles.assign_role(steve, osohq, "member")
    oso.roles.assign_role(leina, oso_repo, "writer")

    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "push", oso_repo)

    assert oso.is_allowed(steve, "pull", oso_repo)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(steve, "push", oso_repo)


# Homogeneous role-permission assignment:
def test_homogeneous_role_perm(init_oso, sample_data):
    # - Adding a permission of same resource type to a role grants assignee access
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)

    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "invite", osohq)

    # - Removing a permission of same resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                # REMOVE INVITE AND ADD LIST_REPOS
                permissions: ["list_repos"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso.roles.config = None
    oso.load_str(new_policy)
    oso.roles.synchronize_data()

    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "list_repos", osohq)
    assert not oso.is_allowed(steve, "list_repos", osohq)


# Parent->child role-permission assignment:
def test_parent_child_role_perm(init_oso, sample_data):
    # - Adding a permission of child resource type to a role grants assignee access
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite", "repo:pull"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ];

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);

    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)

    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert not oso.is_allowed(steve, "pull", oso_repo)

    # - Removing a permission of child resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ];

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso.roles.config = None
    oso.load_str(new_policy)
    oso.roles.synchronize_data()

    assert not oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)


# Grandparent->child role-permission assignment:
def test_grandparent_child_role_perm(init_oso, sample_data):
    # - Adding a permission of grandchild resource type to a role grants assignee access (without intermediate resource)
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["list_repos", "invite"] and
        roles = {
            member: {
                permissions: ["list_repos", "issue:edit"]
            },
            owner: {
                permissions: ["invite"],
                implies: ["member"]
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
    oso.roles.config = None
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_bug = sample_data["oso_bug"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    session.commit()

    assert oso.is_allowed(leina, "list_repos", osohq)
    assert oso.is_allowed(leina, "edit", oso_bug)
    assert not oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    oso.roles.assign_role(steve, osohq, "owner", session=session)
    session.commit()
    assert oso.is_allowed(steve, "edit", oso_bug)
    assert oso.is_allowed(steve, "list_repos", osohq)
    assert oso.is_allowed(steve, "invite", osohq)

    # - Removing a permission of grandchild resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
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

    oso.clear_rules()
    oso.load_str(new_policy)
    oso.roles.config = None
    oso.roles.synchronize_data()

    assert not oso.is_allowed(leina, "edit", oso_bug)
    assert oso.is_allowed(leina, "invite", osohq)


# Homogeneous role implications:
def test_homogeneous_role_implication(init_oso, sample_data):
    # - Adding a role implication of same resource type to a role grants assignee access
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            },
            owner: {
                implies: ["member"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.config = None
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assert not oso.is_allowed(leina, "invite", osohq)

    oso.roles.assign_role(leina, osohq, "member", session=session)
    oso.roles.assign_role(steve, osohq, "owner", session=session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "invite", osohq)

    # - Removing a role implication of same resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                permissions: ["invite"]
            },
            owner: {
                # REMOVE "implies"
                permissions: ["list_repos"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso.load_str(new_policy)
    oso.roles.config = None
    oso.roles.synchronize_data()

    # leina can still "invite"
    assert oso.is_allowed(leina, "invite", osohq)

    # steve can't "invite"
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)


# Parent->child role implications:
def test_parent_child_role_implication(init_oso, sample_data):
    # - Adding a role implication of child resource type to a role grants assignee access to child
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);

    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # member implies reader which has the "pull" permission
    oso.roles.assign_role(leina, osohq, "member", session=session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert not oso.is_allowed(steve, "pull", oso_repo)

    # - Removing a role implication of child resource type from a role revokes assignee access to child
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ];

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso.load_str(new_policy)
    oso.roles.config = None
    oso.roles.synchronize_data()

    assert not oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)


# Grandparent->child role implications:
def test_grandparent_child_role_implication(init_oso, sample_data):
    # - Adding a role implication of grandchild resource type to a role grants assignee access to grandchild
    #   without intermediate parent resource
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["issue:editor"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            editor: {
                permissions: ["edit"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    parent(issue: Issue, parent_repo: Repository) if
        issue.repo = parent_repo;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_bug = sample_data["oso_bug"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "edit", oso_bug)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    # - Removing a permission of grandchild resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            editor: {
                permissions: ["edit"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    parent(issue: Issue, parent_repo: Repository) if
        issue.repo = parent_repo;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso.roles.config = None
    oso.load_str(new_policy)
    oso.roles.synchronize_data()

    assert not oso.is_allowed(leina, "edit", oso_bug)
    assert oso.is_allowed(leina, "invite", osohq)


def test_chained_role_implication(init_oso, sample_data):
    # - Adding a role implication from grandparent->parent->child resource role types grants assignee of grandparent
    # role access to grandchild resource
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]

            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"],
                implies: ["issue:editor"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            editor: {
                permissions: ["edit"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    parent(issue: Issue, parent_repo: Repository) if
        issue.repo = parent_repo;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    oso_bug = sample_data["oso_bug"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    oso.roles.assign_role(steve, oso_repo, "reader", session=session)
    session.commit()

    # leina can invite to the org, pull from the repo, and edit the issue
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert oso.is_allowed(leina, "edit", oso_bug)

    # steve can pull from the repo and edit the issue, but can NOT invite to the org
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert oso.is_allowed(steve, "edit", oso_bug)
    assert not oso.is_allowed(steve, "invite", osohq)

    # - Removing a role implication from grandparent->parent->child resource role types revokes assignee of grandparent
    # role access to grandchild resource
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"],
                implies: ["issue:editor"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            editor: {
                permissions: ["edit"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    parent(issue: Issue, parent_repo: Repository) if
        issue.repo = parent_repo;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso.roles.config = None
    oso.load_str(new_policy)
    oso.roles.synchronize_data()

    # leina can't edit the issue anymore
    assert not oso.is_allowed(leina, "edit", oso_bug)
    assert oso.is_allowed(leina, "invite", osohq)

    # steve can still edit the issue
    assert oso.is_allowed(steve, "edit", oso_bug)


# TEST WRITE API
# User-role assignment:
# - [x] Adding user-role assignment grants access
# - [x] Removing user-role assignment revokes access
# - [x] Assigning/removing non-existent role throws an error
# - [x] Removing user from a role they aren't assigned throws an error
# - [x] Assigning to role with wrong resource type throws an error
# - [x] Reassigning user role throws error if `reassign=False`


def test_assign_role_wrong_resource_type(init_oso, sample_data):
    # - Assigning to role with wrong resource type throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]

    with pytest.raises(OsoError):
        oso.roles.assign_role(leina, oso_repo, "member", session=session)


def test_assign_remove_nonexistent_role(init_oso, sample_data):
    # - Assigning/removing non-existent role throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    with pytest.raises(OsoError):
        oso.roles.assign_role(leina, osohq, "owner", session=session)

    with pytest.raises(OsoError):
        oso.roles.remove_role(leina, osohq, "owner", session=session)


def test_remove_unassigned_role(init_oso, sample_data):
    # - Removing role that user doesn't have returns false
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    removed = oso.roles.remove_role(leina, osohq, "member", session=session)
    assert not removed


def test_assign_remove_user_role(init_oso, sample_data):
    # - Adding user-role assignment grants access
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                permissions: ["invite"]
            },
            owner: {
                permissions: ["list_repos"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    session.commit()

    # Assign leina member role
    leina_roles = (
        session.query(oso.roles.UserRole)
        .filter(oso.roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 1
    assert leina_roles[0].role == "org:member"

    # Assign steve owner role
    oso.roles.assign_role(steve, osohq, "owner", session=session)
    session.commit()

    steve_roles = (
        session.query(oso.roles.UserRole)
        .filter(oso.roles.UserRole.user_id == steve.id)
        .all()
    )
    assert len(steve_roles) == 1
    assert steve_roles[0].role == "org:owner"

    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)

    # - Removing user-role assignment revokes access
    removed = oso.roles.remove_role(leina, osohq, "member", session=session)
    session.commit()
    assert removed
    leina_roles = (
        session.query(oso.roles.UserRole)
        .filter(oso.roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 0

    # make sure steve still has his role
    steve_roles = (
        session.query(oso.roles.UserRole)
        .filter(oso.roles.UserRole.user_id == steve.id)
        .all()
    )
    assert len(steve_roles) == 1
    assert steve_roles[0].role == "org:owner"

    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)


def test_reassign_user_role(init_oso, sample_data):
    # - Implied roles for the same resource type are mutually exclusive on user-role assignment
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                permissions: ["invite"]
            },
            owner: {
                permissions: ["list_repos"],
                implies: ["member", "repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session)
    session.commit()
    leina_roles = (
        session.query(oso.roles.UserRole)
        .filter(oso.roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 1
    assert leina_roles[0].role == "org:member"

    oso.roles.assign_role(steve, osohq, "owner", session)
    session.commit()
    steve_roles = (
        session.query(oso.roles.UserRole)
        .filter(oso.roles.UserRole.user_id == steve.id)
        .all()
    )
    assert len(steve_roles) == 1
    assert steve_roles[0].role == "org:owner"

    # reassigning with reassign=False throws an error
    with pytest.raises(OsoError):
        oso.roles.assign_role(leina, osohq, "owner", reassign=False)

    # reassign with reassign=True
    oso.roles.assign_role(leina, osohq, "owner", session)
    session.commit()

    leina_roles = (
        session.query(oso.roles.UserRole)
        .filter(oso.roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 1
    assert leina_roles[0].role == "org:owner"


# TEST DATA FILTERING
# - [x] `role_allows` with another rule that produces false filter (implicit OR)
# - [x] `role_allows` inside of an `OR` with another expression
# - [x] `role_allows` inside of an `AND` with another expression
# - [x] `role_allows` inside of a `not` (this probably won't work, so need error handling)


def test_authorizing_related_fields(
    init_oso, sample_data, auth_sessionmaker, Organization, Repository
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "read"] and
        roles = {
            member: {
                permissions: ["invite", "read"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    steve = sample_data["steve"]

    oso.roles.assign_role(steve, osohq, "member", session)
    session.commit()

    oso.actor = steve

    oso.checked_permissions = {Repository: "pull"}
    results = auth_sessionmaker().query(Repository).all()
    assert len(results) == 2
    assert results[0].org is None

    oso.checked_permissions = {Organization: "read", Repository: "pull"}
    results = auth_sessionmaker().query(Repository).all()
    assert len(results) == 2
    assert results[0].org.id == osohq.id


def test_data_filtering_role_allows_not(
    init_oso, sample_data, auth_sessionmaker, Organization
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    allow(actor, action, resource) if
        not Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    oso.roles.assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "invite", apple)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    oso.actor = leina
    oso.checked_permissions = {Organization: "invite"}
    auth_session = auth_sessionmaker()

    with pytest.raises(OsoError):
        auth_session.query(Organization).all()


def test_data_filtering_role_allows_and(
    init_oso, sample_data, auth_sessionmaker, User, Organization
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource) and
        resource.id = "osohq";
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    oso.roles.assign_role(leina, apple, "member", session=session)
    oso.roles.assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(leina, "invite", apple)

    oso.actor = leina
    oso.checked_permissions = {Organization: "invite"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 1

    oso.actor = steve
    oso.checked_permissions = {Organization: "invite", User: "invite"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(User).all()
    assert len(results) == 0


def test_data_filtering_role_allows_explicit_or(
    init_oso, sample_data, auth_sessionmaker, User, Organization, Repository
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource) or
        resource.id = "osohq";
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(steve, apple, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    oso.actor = steve
    oso.checked_permissions = {Organization: "invite"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 2

    oso.actor = steve
    oso.checked_permissions = {Repository: "pull"}
    auth_session = auth_sessionmaker()
    results = auth_session.query(Repository).all()
    assert len(results) == 1
    assert results[0].org_id == "apple"

    oso.actor = leina
    oso.checked_permissions = {Organization: "invite", User: "invite"}
    auth_session = auth_sessionmaker()
    results = auth_session.query(Organization).all()
    assert len(results) == 1


def test_data_filtering_role_allows_implicit_or(
    init_oso, sample_data, auth_sessionmaker, User, Organization
):
    # Ensure that the filter produced by `Roles.role_allows()` is not AND-ed
    # with a false filter produced by a separate `allow()` rule.
    oso, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Organization, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            member: {
                permissions: ["read"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", leina)

    oso.actor = leina
    oso.checked_permissions = {Organization: "read", User: "read"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 1

    results = auth_session.query(User).all()
    assert len(results) == 1


def test_data_filtering_user_in_role_not(
    init_oso, sample_data, auth_sessionmaker, Organization
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    allow(actor, action, resource) if
        not Roles.user_in_role(actor, "member", resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    oso.roles.assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "invite", apple)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    oso.actor = leina
    oso.checked_permissions = {Organization: "invite"}
    auth_session = auth_sessionmaker()

    with pytest.raises(OsoError):
        auth_session.query(Organization).all()


def test_data_filtering_user_in_role_and(
    init_oso, sample_data, auth_sessionmaker, User, Organization
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.user_in_role(actor, "member", resource) and
        resource.id = "osohq";
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    oso.roles.assign_role(leina, apple, "member", session=session)
    oso.roles.assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(leina, "invite", apple)

    oso.actor = leina
    oso.checked_permissions = {Organization: "invite"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 1

    oso.actor = steve
    oso.checked_permissions = {User: "invite"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(User).all()
    assert len(results) == 0


def test_data_filtering_user_in_role_explicit_or(
    init_oso, sample_data, auth_sessionmaker, User, Organization, Repository
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);

    allow(actor, _, resource) if
        Roles.user_in_role(actor, "member", resource) or
        resource.id = "osohq";
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso.roles.assign_role(steve, apple, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    oso.actor = steve
    oso.checked_permissions = {Organization: "invite"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 2

    oso.actor = steve
    oso.checked_permissions = {Repository: "pull"}
    auth_session = auth_sessionmaker()
    results = auth_session.query(Repository).all()
    assert len(results) == 1
    assert results[0].org_id == "apple"

    oso.actor = leina
    oso.checked_permissions = {Organization: "invite"}
    auth_session = auth_sessionmaker()
    results = auth_session.query(Organization).all()
    assert len(results) == 1


def test_data_filtering_user_in_role_implicit_or(
    init_oso, sample_data, auth_sessionmaker, User, Organization
):
    # Ensure that the filter produced by `Roles.role_allows()` is not AND-ed
    # with a false filter produced by a separate `allow()` rule.
    oso, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Organization, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            member: {
                permissions: ["read"]
            }
        };

    allow(actor, action, resource) if
        Roles.user_in_role(actor, "member", resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", leina)

    oso.actor = leina
    oso.checked_permissions = {Organization: "read", User: "read"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 1

    results = auth_session.query(User).all()
    assert len(results) == 1


def test_data_filtering_combo(
    init_oso, sample_data, auth_sessionmaker, User, Organization
):
    oso, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Organization, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            member: {
                permissions: ["read"]
            }
        };

    allow(actor, action, resource) if
        role_allows = Roles.role_allows(actor, action, resource) and
        user_in_role = Roles.user_in_role(actor, "member", resource) and
        role_allows and user_in_role;
    """
    # You can't directly `and` the two Roles calls right now but it does work if you do it like ^
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    oso.roles.assign_role(leina, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", leina)

    oso.actor = leina
    oso.checked_permissions = {Organization: "read"}
    auth_session = auth_sessionmaker()

    # TODO: for now this will error
    with pytest.raises(OsoError):
        auth_session.query(Organization).all()


# TEST READ API
# - [ ] Test getting all roles for a resource
# - [ ] Test getting all role assignments for a resource


def test_read_api(init_oso, sample_data, Repository, Organization):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                permissions: ["list_repos"]
            },
            owner: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;


    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    ios = sample_data["ios"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # - [ ] Test getting all roles for a resource
    repo_roles = oso.roles.for_resource(Repository, session)
    assert len(repo_roles) == 1
    assert repo_roles[0] == "reader"

    org_roles = oso.roles.for_resource(Organization, session)
    assert len(org_roles) == 2
    assert "member" in org_roles
    assert "owner" in org_roles

    # - [ ] Test getting all role assignments for a resource
    oso.roles.assign_role(leina, osohq, "member", session=session)
    oso.roles.assign_role(leina, oso_repo, "reader", session=session)

    oso.roles.assign_role(steve, osohq, "owner", session=session)
    oso.roles.assign_role(steve, ios, "reader", session=session)
    session.commit()

    osohq_assignments = oso.roles.assignments_for_resource(osohq, session)
    assert len(osohq_assignments) == 2
    oso_repo_assignments = oso.roles.assignments_for_resource(oso_repo, session)
    assert len(oso_repo_assignments) == 1
    ios_assignments = oso.roles.assignments_for_resource(ios, session)
    assert len(ios_assignments) == 1

    leina_assignments = oso.roles.assignments_for_user(leina, session)
    assert len(leina_assignments) == 2
    steve_assignments = oso.roles.assignments_for_user(steve, session)
    assert len(steve_assignments) == 2


def test_user_in_role(
    init_oso, sample_data, Repository, Organization, auth_sessionmaker
):
    oso, session = init_oso
    policy = """
    resource(_type: Organization, "org", _actions, roles) if
        roles = {
            member: {
                implies: ["repo:reader"]
            },
            owner: {
                implies: ["member"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;


    allow(actor, "read", repo: Repository) if
        Roles.user_in_role(actor, "reader", repo);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]
    gabe = sample_data["gabe"]

    oso.roles.assign_role(leina, osohq, "member")
    oso.roles.assign_role(steve, oso_repo, "reader")

    # Without data filtering
    assert oso.is_allowed(leina, "read", oso_repo)
    assert oso.is_allowed(steve, "read", oso_repo)
    assert not oso.is_allowed(gabe, "read", oso_repo)

    # With data filtering
    oso.actor = leina
    oso.checked_permissions = {Repository: "read"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Repository).all()
    assert len(results) == 2
    for repo in results:
        assert repo.org_id == "osohq"


def test_mismatched_id_types_throws_error(engine, Base, User):
    class One(Base):
        __tablename__ = "ones"

        id = Column(String(), primary_key=True)

    class Two(Base):
        __tablename__ = "twos"

        id = Column(Integer(), primary_key=True)

    Session = sessionmaker(bind=engine)

    oso = SQLAlchemyOso(Base)

    with pytest.raises(OsoError):
        oso.enable_roles(User, Session)


def test_enable_roles_twice(engine, Base, User):
    class One(Base):
        __tablename__ = "ones"

        id = Column(Integer(), primary_key=True)

    Session = sessionmaker(bind=engine)
    oso = SQLAlchemyOso(Base)

    oso.enable_roles(User, Session)

    with pytest.raises(OsoError):
        oso.enable_roles(User, Session)


def test_global_declarative_base(engine, Base, User):
    """Test two different Osos & two different OsoRoles but a shared
    declarative_base(). This shouldn't error."""

    class One(Base):
        __tablename__ = "ones"

        id = Column(Integer(), primary_key=True)

    Session = sessionmaker(bind=engine)
    oso = SQLAlchemyOso(Base)
    oso.enable_roles(User, Session)

    oso2 = SQLAlchemyOso(Base)
    oso2.enable_roles(User, Session)


@pytest.mark.parametrize("sa_type,one_id", [(String, "1"), (Integer, 1)])
def test_id_types(engine, Base, User, sa_type, one_id):
    class One(Base):
        __tablename__ = "ones"

        id = Column(sa_type(), primary_key=True)

    class Two(Base):
        __tablename__ = "twos"

        id = Column(sa_type(), primary_key=True)

    Session = sessionmaker(bind=engine)
    session = Session()

    oso = SQLAlchemyOso(Base)
    oso.enable_roles(User, Session)

    Base.metadata.create_all(engine)

    policy = """
    resource(_type: One, "one", ["read"], {boss: {permissions: ["read"]}});
    resource(_type: Two, "two", ["read"], _roles);

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso.roles.synchronize_data()

    steve = User(name="steve")
    one = One(id=one_id)

    session.add(steve)
    session.add(one)
    session.commit()

    oso.roles.assign_role(steve, one, "boss")
    session.commit()
    assert oso.is_allowed(steve, "read", one)


# LEGACY TEST


def test_roles_integration(
    init_oso, auth_sessionmaker, User, Organization, Repository, Issue
):
    oso, session = init_oso

    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite",
            "create_repo"
        ] and
        roles = {
            member: {
                permissions: ["create_repo"],
                implies: ["repo:reader"]
            },
            owner: {
                permissions: ["invite"],
                implies: ["member", "repo:writer"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            writer: {
                permissions: ["push", "issue:edit"],
                implies: ["reader"]
            },
            reader: {
                permissions: ["pull"]
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
    oso.roles.synchronize_data()

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
    gabe = User(name="gabe")

    objs = [leina, steve, gabe, apple, osohq, ios, oso_repo, demo_repo, laggy, bug]
    for obj in objs:
        session.add(obj)
    session.commit()

    # @NOTE: Need the users and resources in the db before assigning roles
    # so you have to call session.commit() first.
    oso.roles.assign_role(leina, osohq, "owner", session=session)
    oso.roles.assign_role(steve, osohq, "member", session=session)
    session.commit()

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
    oso.checked_permissions = {Repository: "pull"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Repository).all()
    assert len(results) == 2
    result_ids = [repo.id for repo in results]
    assert oso_repo.id in result_ids
    assert demo_repo.id in result_ids
    assert ios.id not in result_ids

    oso.actor = leina
    oso.checked_permissions = {Issue: "edit"}
    auth_session = auth_sessionmaker()

    results = auth_session.query(Issue).all()
    assert len(results) == 1
    result_ids = [issue.id for issue in results]
    assert bug.id in result_ids
    assert not oso.is_allowed(gabe, "edit", bug)
    oso.roles.assign_role(gabe, osohq, "member", session=session)
    session.commit()
    assert not oso.is_allowed(gabe, "edit", bug)
    oso.roles.assign_role(gabe, osohq, "owner", session=session)
    session.commit()
    assert oso.is_allowed(gabe, "edit", bug)
    oso.roles.assign_role(gabe, osohq, "member", session=session)
    session.commit()
    assert not oso.is_allowed(gabe, "edit", bug)
    oso.roles.assign_role(gabe, osohq, "owner", session=session)
    session.commit()
    assert oso.is_allowed(gabe, "edit", bug)
    oso.roles.remove_role(gabe, osohq, "owner", session=session)
    session.commit()
    assert not oso.is_allowed(gabe, "edit", bug)

    org_roles = oso.roles.for_resource(Repository)
    assert set(org_roles) == {"reader", "writer"}
    oso_assignments = oso.roles.assignments_for_resource(osohq)
    assert oso_assignments == [
        {"user_id": leina.id, "role": "owner"},
        {"user_id": steve.id, "role": "member"},
    ]
