# Roles 2 tests
import pytest

from sqlalchemy import create_engine
from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso import register_models, authorized_sessionmaker
from sqlalchemy_oso.roles2 import OsoRoles

from oso import Oso, OsoError


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


@pytest.fixture
def sample_data(init_oso):
    _, _, session = init_oso
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

    objs = {
        "leina": leina,
        "steve": steve,
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


def test_oso_roles_init(auth_sessionmaker):
    oso = Oso()
    register_models(oso, Base)

    # - Passing an auth session to OsoRoles raises an exception
    with pytest.raises(OsoError):
        OsoRoles(oso, Base, User, auth_sessionmaker)

    Session = sessionmaker(bind=engine)
    session = Session()

    # - Passing a session instead of Session factory to OsoRoles raises an exception
    with pytest.raises(AttributeError):
        OsoRoles(oso, Base, User, session)

    class FakeClass:
        pass

    # - Passing a non-SQLAlchemy user model to OsoRoles raises an exception
    with pytest.raises(TypeError):
        OsoRoles(oso, Base, FakeClass, Session)

    # - Passing a bad declarative_base to OsoRoles raises an exception
    with pytest.raises(AttributeError):
        OsoRoles(oso, FakeClass, User, Session)


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
    oso, oso_roles, session = init_oso
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
        oso_roles.synchronize_data()
    pass


def test_bad_namespace_perm(init_oso):
    # - assigning permission with bad namespace throws an error
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                perms: ["repo:pull"]
            }
        };
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso_roles.synchronize_data()
    pass


# TODO
def test_resource_with_roles_no_actions(init_oso, sample_data):
    # - only define roles, no actions (role has actions/implications from different resource)
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", _, roles) if
        roles = {
            member: {
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

        parent(repo: Repository, parent_org: Organization) if
            repo.org = parent_org;

        allow(actor, action, resource) if
            Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)

    oso_roles.synchronize_data()

    leina = sample_data["leina"]
    steve = sample_data["steve"]
    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]

    oso_roles.assign_role(leina, osohq, "member", session)
    oso_roles.assign_role(steve, oso_repo, "repo_read", session)

    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(steve, "pull", oso_repo)


def test_duplicate_resource_name(init_oso):
    # - duplicate resource name throws an error
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                perms: ["invite"]
            }
        };

    # DUPLICATE RESOURCE NAME "org"
    resource(_type: Repository, "org", actions, roles) if
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

    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


def test_nested_dot_relationship(init_oso):
    # - multiple dot lookups throws an error for now
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                perms: ["invite"]
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
        oso_roles.synchronize_data()


def test_bad_relationship_lookup(init_oso):
    # - nonexistent attribute lookup throws an error for now
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                perms: ["invite"]
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
        oso_roles.synchronize_data()


def test_relationship_without_specializer(init_oso):
    oso, oso_roles, session = init_oso
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
        oso_roles.synchronize_data()


def test_relationship_without_resources(init_oso):
    oso, oso_roles, session = init_oso
    policy = """
    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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

    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    oso_roles.synchronize_data()


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

    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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

    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
        repo.org = parent_org;
    """

    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


def test_undefined_resource_arguments(init_oso):
    # - use resource predicate without defining actions/roles
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles);
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    with pytest.raises(OsoError):
        oso_roles.synchronize_data()


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
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"],
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
            },
            repo_write: {
                # repo_write is more permissive than org_member
                perms: ["push"],
                implies: ["repo_read"]
            }
        };

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # repo_write is more permissive than org_member
    oso_roles.assign_role(leina, osohq, "org_member")
    oso_roles.assign_role(steve, osohq, "org_member")
    oso_roles.assign_role(leina, oso_repo, "repo_write")

    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "push", oso_repo)

    assert oso.is_allowed(steve, "pull", oso_repo)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(steve, "push", oso_repo)


# Homogeneous role-permission assignment:
def test_homogeneous_role_perm(init_oso, sample_data):
    # - Adding a permission of same resource type to a role grants assignee access
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)

    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "invite", osohq)

    # - Removing a permission of same resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            org_member: {
                # REMOVE INVITE AND ADD LIST_REPOS
                perms: ["list_repos"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso_roles.configured = False
    oso.load_str(new_policy)
    oso_roles.synchronize_data()

    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "list_repos", osohq)
    assert not oso.is_allowed(steve, "list_repos", osohq)


# Parent->child role-permission assignment:
def test_parent_child_role_perm(init_oso, sample_data):
    # - Adding a permission of child resource type to a role grants assignee access
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite", "repo:pull"]
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
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert not oso.is_allowed(steve, "pull", oso_repo)

    # - Removing a permission of child resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
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
    oso_roles.configured = False
    oso.load_str(new_policy)
    oso_roles.synchronize_data()

    assert not oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)


# Grandparent->child role-permission assignment:
def test_grandparent_child_role_perm(init_oso, sample_data):
    # - Adding a permission of grandchild resource type to a role grants assignee access (without intermediate resource)
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["list_repos", "invite"] and
        roles = {
            org_member: {
                perms: ["list_repos", "issue:edit"]
            },
            org_owner: {
                perms: ["invite"],
                implies: ["org_member"]
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
    oso_roles.configured = False
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_bug = sample_data["oso_bug"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)

    assert oso.is_allowed(leina, "list_repos", osohq)
    assert oso.is_allowed(leina, "edit", oso_bug)
    assert not oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    oso_roles.assign_role(steve, osohq, "org_owner", session=session)
    assert oso.is_allowed(steve, "edit", oso_bug)
    assert oso.is_allowed(steve, "list_repos", osohq)
    assert oso.is_allowed(steve, "invite", osohq)

    # - Removing a permission of grandchild resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
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
    oso_roles.configured = False
    oso_roles.synchronize_data()

    assert not oso.is_allowed(leina, "edit", oso_bug)
    assert oso.is_allowed(leina, "invite", osohq)


# Homogeneous role implications:
def test_homogeneous_role_implication(init_oso, sample_data):
    # - Adding a role implication of same resource type to a role grants assignee access
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            },
            org_owner: {
                implies: ["org_member"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.configured = False
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assert not oso.is_allowed(leina, "invite", osohq)

    oso_roles.assign_role(leina, osohq, "org_member", session=session)
    oso_roles.assign_role(steve, osohq, "org_owner", session=session)

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "invite", osohq)

    # - Removing a role implication of same resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            org_member: {
                perms: ["invite"]
            },
            org_owner: {
                # REMOVE "implies"
                perms: ["list_repos"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """

    oso.clear_rules()
    oso.load_str(new_policy)
    oso_roles.configured = False
    oso_roles.synchronize_data()

    # leina can still "invite"
    assert oso.is_allowed(leina, "invite", osohq)

    # steve can't "invite"
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)


# Parent->child role implications:
def test_parent_child_role_implication(init_oso, sample_data):
    # - Adding a role implication of child resource type to a role grants assignee access to child
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"],
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

    parent(repository: Repository, parent_org: Organization) if
        repository.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);

    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # org_member implies repo_read which has the "pull" permission
    oso_roles.assign_role(leina, osohq, "org_member", session=session)

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert not oso.is_allowed(steve, "pull", oso_repo)

    # - Removing a role implication of child resource type from a role revokes assignee access to child
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
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
    oso_roles.configured = False
    oso_roles.synchronize_data()

    assert not oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)


# Grandparent->child role implications:
def test_grandparent_child_role_implication(init_oso, sample_data):
    # - Adding a role implication of grandchild resource type to a role grants assignee access to grandchild
    #   without intermediate parent resource
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"],
                implies: ["issue_editor"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            issue_editor: {
                perms: ["edit"]
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
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_bug = sample_data["oso_bug"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "edit", oso_bug)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    # - Removing a permission of grandchild resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            issue_editor: {
                perms: ["edit"]
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
    oso_roles.configured = False
    oso.load_str(new_policy)
    oso_roles.synchronize_data()

    assert not oso.is_allowed(leina, "edit", oso_bug)
    assert oso.is_allowed(leina, "invite", osohq)


def test_chained_role_implication(init_oso, sample_data):
    # - Adding a role implication from grandparent->parent->child resource role types grants assignee of grandparent
    # role access to grandchild resource
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"],
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
                perms: ["pull"],
                implies: ["issue_editor"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            issue_editor: {
                perms: ["edit"]
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
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    oso_bug = sample_data["oso_bug"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)
    oso_roles.assign_role(steve, oso_repo, "repo_read", session=session)

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
            org_member: {
                perms: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            repo_read: {
                perms: ["pull"],
                implies: ["issue_editor"]
            }
        };

    resource(_type: Issue, "issue", actions, roles) if
        actions = [
            "edit"
        ] and
        roles = {
            issue_editor: {
                perms: ["edit"]
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
    oso_roles.configured = False
    oso.load_str(new_policy)
    oso_roles.synchronize_data()

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
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]

    with pytest.raises(OsoError):
        oso_roles.assign_role(leina, oso_repo, "org_member", session=session)


def test_assign_remove_nonexistent_role(init_oso, sample_data):
    # - Assigning/removing non-existent role throws an error
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    with pytest.raises(OsoError):
        oso_roles.assign_role(leina, osohq, "org_owner", session=session)

    with pytest.raises(OsoError):
        oso_roles.remove_role(leina, osohq, "org_owner", session=session)


def test_remove_unassigned_role(init_oso, sample_data):
    # - Removing role that user doesn't have returns false
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    removed = oso_roles.remove_role(leina, osohq, "org_member", session=session)
    assert not removed


def test_assign_remove_user_role(init_oso, sample_data):
    # - Adding user-role assignment grants access
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            org_member: {
                perms: ["invite"]
            },
            org_owner: {
                perms: ["list_repos"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)

    # Assign leina member role
    leina_roles = (
        session.query(oso_roles.UserRole)
        .filter(oso_roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 1
    assert leina_roles[0].role == "org_member"

    # Assign steve owner role
    oso_roles.assign_role(steve, osohq, "org_owner", session=session)

    steve_roles = (
        session.query(oso_roles.UserRole)
        .filter(oso_roles.UserRole.user_id == steve.id)
        .all()
    )
    assert len(steve_roles) == 1
    assert steve_roles[0].role == "org_owner"

    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)

    # - Removing user-role assignment revokes access
    removed = oso_roles.remove_role(leina, osohq, "org_member", session=session)
    assert removed
    leina_roles = (
        session.query(oso_roles.UserRole)
        .filter(oso_roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 0

    # make sure steve still has his role
    steve_roles = (
        session.query(oso_roles.UserRole)
        .filter(oso_roles.UserRole.user_id == steve.id)
        .all()
    )
    assert len(steve_roles) == 1
    assert steve_roles[0].role == "org_owner"

    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)


def test_reassign_user_role(init_oso, sample_data):
    # - Implied roles for the same resource type are mutually exclusive on user-role assignment
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            org_member: {
                perms: ["invite"]
            },
            org_owner: {
                perms: ["list_repos"],
                implies: ["org_member", "repo_read"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            repo_read: {
                perms: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session)
    leina_roles = (
        session.query(oso_roles.UserRole)
        .filter(oso_roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 1
    assert leina_roles[0].role == "org_member"

    oso_roles.assign_role(steve, osohq, "org_owner", session)
    steve_roles = (
        session.query(oso_roles.UserRole)
        .filter(oso_roles.UserRole.user_id == steve.id)
        .all()
    )
    assert len(steve_roles) == 1
    assert steve_roles[0].role == "org_owner"

    # reassigning with reassign=False throws an error
    with pytest.raises(OsoError):
        oso_roles.assign_role(leina, osohq, "org_owner", reassign=False)

    # reassign with reassign=True
    oso_roles.assign_role(leina, osohq, "org_owner", session)

    leina_roles = (
        session.query(oso_roles.UserRole)
        .filter(oso_roles.UserRole.user_id == leina.id)
        .all()
    )
    assert len(leina_roles) == 1
    assert leina_roles[0].role == "org_owner"


# TEST DATA FILTERING
# - [x] `role_allows` with another rule that produces false filter (implicit OR)
# - [x] `role_allows` inside of an `OR` with another expression
# - [x] `role_allows` inside of an `AND` with another expression
# - [x] `role_allows` inside of a `not` (this probably won't work, so need error handling)


def test_data_filtering_not(init_oso, sample_data, auth_sessionmaker):
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        not Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)
    oso_roles.assign_role(steve, osohq, "org_member", session=session)

    # This is just to ensure we don't modify the policy above.
    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "invite", apple)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    oso.actor = leina
    oso.action = "invite"
    auth_session = auth_sessionmaker()

    with pytest.raises(OsoError):
        results = auth_session.query(Organization).all()


def test_data_filtering_and(init_oso, sample_data, auth_sessionmaker):
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource) and
        resource.id = "osohq";
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)
    oso_roles.assign_role(leina, apple, "org_member", session=session)
    oso_roles.assign_role(steve, osohq, "org_member", session=session)

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(leina, "invite", apple)

    oso.actor = leina
    oso.action = "invite"
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 1

    oso.actor = steve
    oso.action = "invite"
    auth_session = auth_sessionmaker()

    results = auth_session.query(User).all()
    assert len(results) == 0


def test_data_filtering_explicit_or(init_oso, sample_data, auth_sessionmaker):
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            org_member: {
                perms: ["invite"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource) or
        resource.id = "osohq";
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    oso_roles.assign_role(steve, apple, "org_member", session=session)

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    oso.actor = steve
    oso.action = "invite"
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 2

    oso.actor = leina
    oso.action = "invite"
    auth_session = auth_sessionmaker()

    results = auth_session.query(User).all()
    assert len(results) == 1


def test_data_filtering_implicit_or(init_oso, sample_data, auth_sessionmaker):
    # Ensure that the filter produced by `Roles.role_allows()` is not AND-ed
    # with a false filter produced by a separate `allow()` rule.
    oso, oso_roles, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Organization, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            org_member: {
                perms: ["read"]
            }
        };

    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    oso_roles.assign_role(leina, osohq, "org_member", session=session)

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", leina)

    oso.actor = leina
    oso.action = "read"
    auth_session = auth_sessionmaker()

    results = auth_session.query(Organization).all()
    assert len(results) == 1

    results = auth_session.query(User).all()
    assert len(results) == 1


# TEST READ API
# - [ ] Test getting all roles for a resource
# - [ ] Test getting all role assignments for a resource


def test_read_api(init_oso, sample_data):
    oso, oso_roles, session = init_oso
    policy = """
    resource(_type: Organization, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            org_member: {
                perms: ["list_repos"]
            },
            org_owner: {
                perms: ["invite"]
            }
        };

    resource(_type: Repository, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            repo_read: {
                perms: ["pull"]
            }
        };

    parent(repo: Repository, parent_org: Organization) if
        repo.org = parent_org;


    allow(actor, action, resource) if
        Roles.role_allows(actor, action, resource);
    """
    oso.load_str(policy)
    oso_roles.synchronize_data()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    ios = sample_data["ios"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # - [ ] Test getting all roles for a resource
    repo_roles = oso_roles.for_resource(Repository, session)
    assert len(repo_roles) == 1
    assert repo_roles[0] == "repo_read"

    org_roles = oso_roles.for_resource(Organization, session)
    assert len(org_roles) == 2
    assert "org_member" in org_roles
    assert "org_owner" in org_roles

    # - [ ] Test getting all role assignments for a resource
    oso_roles.assign_role(leina, osohq, "org_member", session=session)
    oso_roles.assign_role(leina, oso_repo, "repo_read", session=session)

    oso_roles.assign_role(steve, osohq, "org_owner", session=session)
    oso_roles.assign_role(steve, ios, "repo_read", session=session)

    osohq_assignments = oso_roles.assignments_for_resource(osohq, session)
    assert len(osohq_assignments) == 2
    oso_repo_assignments = oso_roles.assignments_for_resource(oso_repo, session)
    assert len(oso_repo_assignments) == 1
    ios_assignments = oso_roles.assignments_for_resource(ios, session)
    assert len(ios_assignments) == 1


# LEGACY TEST


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
    oso_roles.synchronize_data()

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

    assert not oso.is_allowed(gabe, "edit", bug)
    oso_roles.assign_role(gabe, osohq, "org_member", session=session)
    assert not oso.is_allowed(gabe, "edit", bug)
    oso_roles.assign_role(gabe, osohq, "org_owner", session=session)
    assert oso.is_allowed(gabe, "edit", bug)
    oso_roles.assign_role(gabe, osohq, "org_member", session=session)
    assert not oso.is_allowed(gabe, "edit", bug)
    oso_roles.assign_role(gabe, osohq, "org_owner", session=session)
    assert oso.is_allowed(gabe, "edit", bug)
    oso_roles.remove_role(gabe, osohq, "org_owner", session=session)
    assert not oso.is_allowed(gabe, "edit", bug)

    org_roles = oso_roles.for_resource(Repository)
    assert set(org_roles) == {"repo_read", "repo_write"}
    oso_assignments = oso_roles.assignments_for_resource(osohq)
    assert oso_assignments == [
        {"user_id": leina.id, "role": "org_owner"},
        {"user_id": steve.id, "role": "org_member"},
    ]
