# Roles 2 tests
import pytest
import timeit
import os

from sqlalchemy import create_engine
from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from oso import Oso, OsoError
from .polar_roles_sqlalchemy_helpers import (
    resource_role_class,
    assign_role,
    remove_role,
)


Base = declarative_base(name="RoleBase")


class Org(Base):  # type: ignore
    __tablename__ = "orgs"

    name = Column(String(), primary_key=True)
    base_repo_role = Column(String())

    def __repr__(self):
        return f"Org({self.name})"


class User(Base):  # type: ignore
    __tablename__ = "users"

    name = Column(String(), primary_key=True)

    def __repr__(self):
        return f"User({self.name})"


class Repo(Base):  # type: ignore
    __tablename__ = "repos"

    repo_id = Column(Integer, primary_key=True)
    name = Column(String(256))

    # many-to-one relationship with orgs
    org_id = Column(Integer, ForeignKey("orgs.name"))
    org = relationship("Org", backref="repos", lazy=True)  # type: ignore

    def __repr__(self):
        return f"Repo({self.name}) <- {self.org}"


class Issue(Base):  # type: ignore
    __tablename__ = "issues"

    issue_id = Column(Integer, primary_key=True)
    name = Column(String(256))
    repo_id = Column(Integer, ForeignKey("repos.repo_id"))
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
    # ---------------------------
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)

    Session = sessionmaker(bind=engine)
    session = Session()

    oso = Oso()

    for m in Base.registry.mappers:
        oso.register_class(m.class_)

    oso.enable_roles()

    return (oso, session)


@pytest.fixture
def sample_data(init_oso):
    _, session = init_oso
    # Create sample data
    # -------------------
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


# TEST OsoRoles Initialization
# - Passing an auth session to OsoRoles raises an exception
# - Passing a session instead of Session factory to OsoRoles raises an exception
# - Passing a non-SQLAlchemy user model to OsoRoles raises an exception
# - Passing a bad declarative_base to OsoRoles raises an exception


@pytest.mark.skip("need to create method for initializing polar roles")
def test_oso_roles_init():
    pass


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
    oso, _ = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {}
        };
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.validate_config()


def test_bad_namespace_perm(init_oso):
    # - assigning permission with bad namespace throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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
        oso.validate_config()


def test_resource_with_roles_no_actions(init_oso, sample_data):
    # - only define roles, no actions (role has actions/implications from different resource)
    oso, session = init_oso
    policy = """
        resource(_type: Org, "org", _, roles) if
            roles = {
                member: {
                    implies: ["repo:reader"]
                }
            };

        resource(_type: Repo, "repo", actions, roles) if
            actions = [
                "push",
                "pull"
            ] and
            roles = {
                reader: {
                    permissions: ["pull"]
                }
            };

        parent(repo: Repo, parent_org) if
            repo.org = parent_org and
            parent_org matches Org;

        actor_role(actor, role) if
            role in actor.repo_roles or
            role in actor.org_roles;

        allow(actor, action, resource) if
            role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    # TODO: where are we going to do config validation?
    # Maybe when user loads their policy file?
    # oso.roles.synchronize_data()

    leina = sample_data["leina"]
    steve = sample_data["steve"]
    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]

    assign_role(leina, osohq, "member", session)
    assign_role(steve, oso_repo, "reader", session)

    session.commit()

    assert oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(steve, "pull", oso_repo)


def test_duplicate_resource_name(init_oso):
    # - duplicate resource name throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    # DUPLICATE RESOURCE NAME "org"
    resource(_type: Repo, "org", actions, roles) if
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
        oso.validate_config()


@pytest.mark.skip("No longer an error")
def test_nested_dot_relationship(init_oso):
    # - multiple dot lookups throws an error for now
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.validate_config()


@pytest.mark.skip("can't detect, not sqlalchemy")
def test_bad_relationship_lookup(init_oso):
    # - nonexistent attribute lookup throws an error for now
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repo, "repo", actions, {}) if
        actions = [
            "pull"
        ];

    parent(repo: Repo, parent_org) if
        # INCORRECT FIELD NAME
        repo.organization = parent_org and
        parent_org matches Org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.validate_config()


@pytest.mark.skip("Can't really test relationship types")
def test_relationship_without_specializer(init_oso):
    oso, session = init_oso
    policy = """
    resource(_type: Repo, "repo", actions, {}) if
        actions = [
            "pull"
        ];

    parent(repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.validate_config()


def test_relationship_without_resources(init_oso):
    oso, session = init_oso
    policy = """
    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.validate_config()


def test_duplicate_role_name_same_resource(init_oso):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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
        oso.validate_config()


def test_role_namespaces(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = [
            "invite", "create_repo"
        ] and
        roles = {
            owner: {
                permissions: ["invite"],
                implies: ["member", "repo:reader"]
            },
            member: {
                permissions: ["create_repo"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]
    gabe = sample_data["gabe"]

    assign_role(leina, osohq, "owner", session)
    assign_role(steve, oso_repo, "reader", session)
    assign_role(gabe, osohq, "member", session)
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
    resource(_type: Org, "org", actions, {}) if
        actions = [
            "invite"
        ];
    """
    oso.load_str(policy)
    oso.validate_config()


def test_duplicate_action(init_oso):
    # - duplicate action
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = [
            "invite",
            "invite"
        ];
    """
    oso.load_str(policy)

    with pytest.raises(OsoError):
        oso.validate_config()


def test_undeclared_permission(init_oso):
    # - assign permission that wasn't declared
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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
        oso.validate_config()


def test_undeclared_role(init_oso):
    # - imply role that wasn't declared
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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
        oso.validate_config()


@pytest.mark.skip("Can't really test relationship types")
def test_role_implication_without_relationship(init_oso):
    # - imply role without valid relationship
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
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
        oso.validate_config()


@pytest.mark.skip("Can't really test relationship types")
def test_role_permission_without_relationship(init_oso):
    # - assign permission without valid relationship
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                permissions: ["repo:push"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ];
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.validate_config()


def test_invalid_role_permission(init_oso):
    # assigning permission on related role type errors if role exists for permission resource
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = [
            "invite"
        ] and
        roles = {
            member: {
                # THIS IS NOT ALLOWED
                permissions: ["repo:push"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["push"]
            }

        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;
    """

    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.validate_config()


def test_permission_assignment_to_implied_role(init_oso):
    # assigning the same permission to two roles where one implies the other throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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
        oso.validate_config()


def test_incorrect_arity_resource(init_oso):
    # - use resource predicate with incorrect arity
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions) if
        actions = [
            "invite"
        ];
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.validate_config()


def test_undefined_resource_arguments(init_oso):
    # - use resource predicate without defining actions/roles
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles);
    """
    oso.load_str(policy)
    with pytest.raises(OsoError):
        oso.validate_config()


def test_wrong_type_resource_arguments(init_oso):
    # - use resource predicate with field types
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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
        oso.validate_config()


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
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
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

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # writer is more permissive than member
    assign_role(leina, osohq, "member", session=session)
    assign_role(steve, osohq, "member", session=session)
    assign_role(leina, oso_repo, "writer", session=session)

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
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)

    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "invite", osohq)

    # - Removing a permission of same resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                # REMOVE INVITE AND ADD LIST_REPOS
                permissions: ["list_repos"]
            }
        };

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """

    oso.clear_rules()
    oso.enable_roles()
    oso.load_str(new_policy)
    oso.validate_config()

    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "list_repos", osohq)
    assert not oso.is_allowed(steve, "list_repos", osohq)


# Parent->child role-permission assignment:
def test_parent_child_role_perm(init_oso, sample_data):
    # - Adding a permission of child resource type to a role grants assignee access
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite", "repo:pull"]
            }
        };

    resource(_type: Repo, "repo", actions, {}) if
        actions = [
            "push",
            "pull"
        ];

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    ios = sample_data["ios"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)

    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert not oso.is_allowed(leina, "pull", ios)
    assert not oso.is_allowed(steve, "pull", oso_repo)

    # - Removing a permission of child resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repo, "repo", actions, {}) if
        actions = [
            "push",
            "pull"
        ];

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """

    oso.clear_rules()
    oso.enable_roles()
    oso.load_str(new_policy)
    oso.validate_config()

    assert not oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)


# Grandparent->child role-permission assignment:
def test_grandparent_child_role_perm(init_oso, sample_data):
    # - Adding a permission of grandchild resource type to a role grants assignee access (without intermediate resource)
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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

    resource(_type: Issue, "issue", actions, {}) if
        actions = [
            "edit"
        ];

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    parent(issue: Issue, parent_repo) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_bug = sample_data["oso_bug"]
    ios_laggy = sample_data["ios_laggy"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    session.commit()

    assert oso.is_allowed(leina, "list_repos", osohq)
    assert oso.is_allowed(leina, "edit", oso_bug)
    assert not oso.is_allowed(leina, "edit", ios_laggy)
    assert not oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    assign_role(steve, osohq, "owner", session=session)
    session.commit()
    assert oso.is_allowed(steve, "edit", oso_bug)
    assert oso.is_allowed(steve, "list_repos", osohq)
    assert oso.is_allowed(steve, "invite", osohq)

    # - Removing a permission of grandchild resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Issue, "issue", actions, {}) if
        actions = [
            "edit"
        ];

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    parent(issue: Issue, parent_repo) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """

    oso.clear_rules()
    oso.enable_roles()
    oso.load_str(new_policy)
    oso.validate_config()

    assert not oso.is_allowed(leina, "edit", oso_bug)
    assert oso.is_allowed(leina, "invite", osohq)


# Homogeneous role implications:
def test_homogeneous_role_implication(init_oso, sample_data):
    # - Adding a role implication of same resource type to a role grants assignee access
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            },
            owner: {
                implies: ["member"]
            }
        };

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assert not oso.is_allowed(leina, "invite", osohq)

    assign_role(leina, osohq, "member", session=session)
    assign_role(steve, osohq, "owner", session=session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(leina, "invite", apple)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(steve, "invite", apple)

    # - Removing a role implication of same resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Org, "org", actions, roles) if
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

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """

    oso.clear_rules()
    oso.enable_roles()
    oso.load_str(new_policy)
    oso.validate_config()

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
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = [
            "push",
            "pull"
        ] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    ios = sample_data["ios"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # member implies reader which has the "pull" permission
    assign_role(leina, osohq, "member", session=session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "pull", oso_repo)
    assert not oso.is_allowed(leina, "pull", ios)
    assert not oso.is_allowed(steve, "pull", oso_repo)

    # - Removing a role implication of child resource type from a role revokes assignee access to child
    new_policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repo, "repo", actions, {}) if
        actions = [
            "push",
            "pull"
        ];

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """

    oso.clear_rules()
    oso.enable_roles()
    oso.load_str(new_policy)
    oso.validate_config()

    assert not oso.is_allowed(leina, "pull", oso_repo)
    assert oso.is_allowed(leina, "invite", osohq)


# Grandparent->child role implications:
def test_grandparent_child_role_implication(init_oso, sample_data):
    # - Adding a role implication of grandchild resource type to a role grants assignee access to grandchild
    #   without intermediate parent resource
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    parent(issue: Issue, parent_repo) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_bug = sample_data["oso_bug"]
    ios_laggy = sample_data["ios_laggy"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    session.commit()

    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "edit", oso_bug)
    assert not oso.is_allowed(leina, "edit", ios_laggy)
    assert not oso.is_allowed(steve, "edit", oso_bug)

    # - Removing a permission of grandchild resource type from a role revokes assignee access
    new_policy = """
    resource(_type: Org, "org", actions, roles) if
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

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    parent(issue: Issue, parent_repo) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """

    oso.clear_rules()
    oso.enable_roles()
    oso.load_str(new_policy)
    oso.validate_config()

    assert not oso.is_allowed(leina, "edit", oso_bug)
    assert oso.is_allowed(leina, "invite", osohq)


def test_chained_role_implication(init_oso, sample_data):
    # - Adding a role implication from grandparent->parent->child resource role types grants assignee of grandparent
    # role access to grandchild resource
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]

            }
        };

    resource(_type: Repo, "repo", actions, roles) if
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

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    parent(issue: Issue, parent_repo) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    oso_bug = sample_data["oso_bug"]
    ios_laggy = sample_data["ios_laggy"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    assign_role(steve, oso_repo, "reader", session=session)
    session.commit()

    # leina can invite to the org, pull from the repo, and edit the issue
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert oso.is_allowed(leina, "edit", oso_bug)
    assert not oso.is_allowed(leina, "edit", ios_laggy)

    # steve can pull from the repo and edit the issue, but can NOT invite to the org
    assert oso.is_allowed(steve, "pull", oso_repo)
    assert oso.is_allowed(steve, "edit", oso_bug)
    assert not oso.is_allowed(steve, "edit", ios_laggy)
    assert not oso.is_allowed(steve, "invite", osohq)

    # - Removing a role implication from grandparent->parent->child resource role types revokes assignee of grandparent
    # role access to grandchild resource
    new_policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
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

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    parent(issue: Issue, parent_repo) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """

    oso.clear_rules()
    oso.enable_roles()
    oso.load_str(new_policy)
    oso.validate_config()

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


@pytest.mark.skip("TODO: validation / not doing management anymore")
def test_assign_role_wrong_resource_type(init_oso, sample_data):
    # - Assigning to role with wrong resource type throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            writer: {
                permissions: ["invite"]
            }
        };
    """
    oso.load_str(policy)
    oso.validate_config()

    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]

    with pytest.raises(OsoError):
        assign_role(leina, oso_repo, "writer", session=session)


@pytest.mark.skip("TODO: validation / not doing management anymore")
def test_assign_remove_nonexistent_role(init_oso, sample_data):
    # - Assigning/removing non-existent role throws an error
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    with pytest.raises(OsoError):
        assign_role(leina, osohq, "owner", session=session)

    with pytest.raises(OsoError):
        remove_role(leina, osohq, "owner", session=session)


# TODO: this is just testing our own code / we don't handle role management
# anymore
def test_remove_unassigned_role(init_oso, sample_data):
    # - Removing role that user doesn't have returns false
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    removed = remove_role(leina, osohq, "member", session=session)
    assert not removed


# TODO: this is just testing our own code / we don't handle role management
# anymore
def test_assign_remove_user_role(init_oso, sample_data):
    # - Adding user-role assignment grants access
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                permissions: ["invite"]
            },
            owner: {
                permissions: ["list_repos"]
            }
        };

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    session.commit()

    # Assign leina member role
    leina_roles = session.query(OrgRole).filter_by(user_id=leina.name).all()
    assert len(leina_roles) == 1
    assert leina_roles[0].name == "member"

    # Assign steve owner role
    assign_role(steve, osohq, "owner", session=session)
    session.commit()

    steve_roles = session.query(OrgRole).filter_by(user_id=steve.name).all()
    assert len(steve_roles) == 1
    assert steve_roles[0].name == "owner"

    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)

    # - Removing user-role assignment revokes access
    removed = remove_role(leina, osohq, "member", session=session)
    session.commit()
    assert removed
    leina_roles = session.query(OrgRole).filter_by(user_id=leina.name).all()
    assert len(leina_roles) == 0

    # make sure steve still has his role
    steve_roles = session.query(OrgRole).filter_by(user_id=steve.name).all()
    assert len(steve_roles) == 1
    assert steve_roles[0].name == "owner"

    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "list_repos", osohq)


# TODO: this is just testing our own code / we don't handle role management
# anymore
def test_reassign_user_role(init_oso, sample_data):
    # - Implied roles for the same resource type are mutually exclusive on user-role assignment
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
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

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session)
    session.commit()
    leina_roles = session.query(OrgRole).filter_by(user_id=leina.name).all()
    assert len(leina_roles) == 1
    assert leina_roles[0].name == "member"

    assign_role(steve, osohq, "owner", session)
    session.commit()
    steve_roles = session.query(OrgRole).filter_by(user_id=steve.name).all()
    assert len(steve_roles) == 1
    assert steve_roles[0].name == "owner"

    # reassigning with reassign=False throws an error
    with pytest.raises(OsoError):
        assign_role(leina, osohq, "owner", session=session, reassign=False)

    # reassign with reassign=True
    assign_role(leina, osohq, "owner", session)
    session.commit()

    leina_roles = session.query(OrgRole).filter_by(user_id=leina.name).all()
    assert len(leina_roles) == 1
    assert leina_roles[0].name == "owner"


# TEST DATA FILTERING
# - [x] `role_allows` with another rule that produces false filter (implicit OR)
# - [x] `role_allows` inside of an `OR` with another expression
# - [x] `role_allows` inside of an `AND` with another expression
# - [x] `role_allows` inside of a `not` (this probably won't work, so need error handling)


@pytest.mark.skip("not worrying about data filtering yet")
def test_authorizing_related_fields(
    init_oso, sample_data, auth_sessionmaker, Org, Repo
):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite", "read"] and
        roles = {
            member: {
                permissions: ["invite", "read"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    steve = sample_data["steve"]

    assign_role(steve, osohq, "member", session)
    session.commit()

    oso.actor = steve

    oso.checked_permissions = {Repo: "pull"}
    results = auth_sessionmaker().query(Repo).all()
    assert len(results) == 2
    assert results[0].org is None

    oso.checked_permissions = {Org: "read", Repo: "pull"}
    results = auth_sessionmaker().query(Repo).all()
    assert len(results) == 2
    assert results[0].org.id == osohq.id


# TODO(gj): data filtering
def test_data_filtering_role_allows_not(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        not role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "invite", apple)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    # oso.actor = leina
    # oso.checked_permissions = {Org: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # with pytest.raises(OsoError):
    #     auth_session.query(Org).all()


# TODO(gj): data filtering
def test_data_filtering_role_allows_and(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    allow(actor, action, resource) if
        role_allow(actor, action, resource) and
        resource.name = "osohq";

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    assign_role(leina, apple, "member", session=session)
    assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(leina, "invite", apple)

    # oso.actor = leina
    # oso.checked_permissions = {Org: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(Org).all()
    # assert len(results) == 1
    #
    # oso.actor = steve
    # oso.checked_permissions = {Org: "invite", User: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(User).all()
    # assert len(results) == 0


# TODO(gj): data filtering
def test_data_filtering_role_allows_explicit_or(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    allow(actor, action, resource) if
        role_allow(actor, action, resource) or
        resource.name = "osohq";

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    # leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(steve, apple, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    # oso.actor = steve
    # oso.checked_permissions = {Org: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(Org).all()
    # assert len(results) == 2
    #
    # oso.actor = steve
    # oso.checked_permissions = {Repo: "pull"}
    # auth_session = auth_sessionmaker()
    # results = auth_session.query(Repo).all()
    # assert len(results) == 1
    # assert results[0].org_id == "apple"
    #
    # oso.actor = leina
    # oso.checked_permissions = {Org: "invite", User: "invite"}
    # auth_session = auth_sessionmaker()
    # results = auth_session.query(Org).all()
    # assert len(results) == 1


# TODO(gj): data filtering
def test_data_filtering_role_allows_implicit_or(init_oso, sample_data):
    # Ensure that the filter produced by `Roles.role_allows()` is not AND-ed
    # with a false filter produced by a separate `allow()` rule.
    oso, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Org, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            member: {
                permissions: ["read"]
            }
        };

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    assign_role(leina, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", leina)

    # oso.actor = leina
    # oso.checked_permissions = {Org: "read", User: "read"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(Org).all()
    # assert len(results) == 1
    #
    # results = auth_session.query(User).all()
    # assert len(results) == 1


# TODO(gj): data filtering
def test_data_filtering_user_in_role_not(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"]
            }
        };

    allow(actor, action, resource) if
        not user_in_role(actor, "member", resource);

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert not oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(leina, "invite", apple)
    assert not oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    # oso.actor = leina
    # oso.checked_permissions = {Org: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # with pytest.raises(OsoError):
    #     auth_session.query(Org).all()


# TODO(gj): data filtering
def test_data_filtering_user_in_role_and(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    allow(actor, action, resource) if
        user_in_role(actor, "member", resource) and
        resource.name = "osohq";

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(leina, osohq, "member", session=session)
    assign_role(leina, apple, "member", session=session)
    assign_role(steve, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "invite", osohq)
    assert oso.is_allowed(steve, "invite", osohq)
    assert not oso.is_allowed(leina, "invite", apple)

    # oso.actor = leina
    # oso.checked_permissions = {Org: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(Org).all()
    # assert len(results) == 1
    #
    # oso.actor = steve
    # oso.checked_permissions = {User: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(User).all()
    # assert len(results) == 0


# TODO(gj): data filtering
def test_data_filtering_user_in_role_explicit_or(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite"] and
        roles = {
            member: {
                permissions: ["invite"],
                implies: ["repo:reader"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);

    allow(actor, _, resource) if
        user_in_role(actor, "member", resource) or
        resource.name = "osohq";

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]
    # leina = sample_data["leina"]
    steve = sample_data["steve"]

    assign_role(steve, apple, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(steve, "invite", osohq)
    assert oso.is_allowed(steve, "invite", apple)

    # oso.actor = steve
    # oso.checked_permissions = {Org: "invite"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(Org).all()
    # assert len(results) == 2
    #
    # oso.actor = steve
    # oso.checked_permissions = {Repo: "pull"}
    # auth_session = auth_sessionmaker()
    # results = auth_session.query(Repo).all()
    # assert len(results) == 1
    # assert results[0].org_id == "apple"
    #
    # oso.actor = leina
    # oso.checked_permissions = {Org: "invite"}
    # auth_session = auth_sessionmaker()
    # results = auth_session.query(Org).all()
    # assert len(results) == 1


# TODO(gj): data filtering
def test_data_filtering_user_in_role_implicit_or(init_oso, sample_data):
    # Ensure that the filter produced by `user_in_role/3` is not AND-ed
    # with a false filter produced by a separate `allow()` rule.
    oso, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Org, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            member: {
                permissions: ["read"]
            }
        };

    allow(actor, _, resource) if
        user_in_role(actor, "member", resource);

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    assign_role(leina, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", leina)

    # oso.actor = leina
    # oso.checked_permissions = {Org: "read", User: "read"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(Org).all()
    # assert len(results) == 1
    #
    # results = auth_session.query(User).all()
    # assert len(results) == 1


# TODO(gj): data filtering
def test_data_filtering_combo(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Org, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            member: {
                permissions: ["read"]
            }
        };

    allow(actor, action, resource) if
        role_allow(actor, action, resource) and
        user_in_role(actor, "member", resource);

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    assign_role(leina, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", leina)

    # oso.actor = leina
    # oso.checked_permissions = {Org: "read"}
    # auth_session = auth_sessionmaker()
    #
    # # TODO: for now this will error
    # with pytest.raises(OsoError):
    #     auth_session.query(Org).all()


# TEST READ API
# - [ ] Test getting all roles for a resource
# - [ ] Test getting all role assignments for a resource


@pytest.mark.skip("TODO: not handling role management anymore")
def test_read_api(init_oso, sample_data, Repo, Org):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", actions, roles) if
        actions = ["invite", "list_repos"] and
        roles = {
            member: {
                permissions: ["list_repos"]
            },
            owner: {
                permissions: ["invite"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    ios = sample_data["ios"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]

    # - [ ] Test getting all roles for a resource
    repo_roles = oso.roles.for_resource(Repo, session)
    assert len(repo_roles) == 1
    assert repo_roles[0] == "reader"

    org_roles = oso.roles.for_resource(Org, session)
    assert len(org_roles) == 2
    assert "member" in org_roles
    assert "owner" in org_roles

    # - [ ] Test getting all role assignments for a resource
    assign_role(leina, osohq, "member", session=session)
    assign_role(leina, oso_repo, "reader", session=session)

    assign_role(steve, osohq, "owner", session=session)
    assign_role(steve, ios, "reader", session=session)
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


# TODO(gj): data filtering
def test_user_in_role(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    resource(_type: Org, "org", [], roles) if
        roles = {
            member: {
                implies: ["repo:reader"]
            },
            owner: {
                implies: ["member"]
            }
        };

    resource(_type: Repo, "repo", actions, roles) if
        actions = ["pull"] and
        roles = {
            reader: {
                permissions: ["pull"]
            }
        };

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    allow(actor, "read", repo: Repo) if
        user_in_role(actor, "reader", repo);

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    oso_repo = sample_data["oso_repo"]
    leina = sample_data["leina"]
    steve = sample_data["steve"]
    gabe = sample_data["gabe"]

    assign_role(leina, osohq, "member", session)
    assign_role(steve, oso_repo, "reader", session)

    # Without data filtering
    assert oso.is_allowed(leina, "read", oso_repo)
    assert oso.is_allowed(steve, "read", oso_repo)
    assert not oso.is_allowed(gabe, "read", oso_repo)

    # # With data filtering
    # oso.actor = leina
    # oso.checked_permissions = {Repo: "read"}
    # auth_session = auth_sessionmaker()
    #
    # results = auth_session.query(Repo).all()
    # assert len(results) == 2
    # for repo in results:
    #     assert repo.org_id == "osohq"


@pytest.mark.skip("TODO: validation / this isn't our problem anymore")
def test_mismatched_id_types_throws_error(engine, Base, User):
    pass
    # class One(Base):
    #     __tablename__ = "ones"
    #
    #     id = Column(String(), primary_key=True)
    #
    # class Two(Base):
    #     __tablename__ = "twos"
    #
    #     id = Column(Integer(), primary_key=True)
    #
    # Session = sessionmaker(bind=engine)
    #
    # oso = SQLAlchemyOso(Base)
    #
    # with pytest.raises(OsoError):
    #     oso.enable_roles(User, Session)


@pytest.mark.skip("TODO: not a thing anymore, maybe?")
def test_enable_roles_twice(engine, Base, User):
    pass
    # class One(Base):
    #     __tablename__ = "ones"
    #
    #     id = Column(Integer(), primary_key=True)
    #
    # Session = sessionmaker(bind=engine)
    # oso = SQLAlchemyOso(Base)
    #
    # oso.enable_roles(User, Session)
    #
    # with pytest.raises(OsoError):
    #     oso.enable_roles(User, Session)


@pytest.mark.skip("TODO: not our problem anymore")
def test_global_declarative_base(engine, Base, User):
    """Test two different Osos & two different OsoRoles but a shared
    declarative_base(). This shouldn't error."""

    pass
    # class One(Base):
    #     __tablename__ = "ones"
    #
    #     id = Column(Integer(), primary_key=True)
    #
    # Session = sessionmaker(bind=engine)
    # oso = SQLAlchemyOso(Base)
    # oso.enable_roles(User, Session)
    #
    # oso2 = SQLAlchemyOso(Base)
    # oso2.enable_roles(User, Session)


@pytest.mark.skip("TODO: not our problem anymore")
@pytest.mark.parametrize("sa_type,one_id", [(String, "1"), (Integer, 1)])
def test_id_types(engine, Base, User, sa_type, one_id):
    pass
    # class One(Base):
    #     __tablename__ = "ones"
    #
    #     id = Column(sa_type(), primary_key=True)
    #
    # class Two(Base):
    #     __tablename__ = "twos"
    #
    #     id = Column(sa_type(), primary_key=True)
    #
    # Session = sessionmaker(bind=engine)
    # session = Session()
    #
    # oso = SQLAlchemyOso(Base)
    # oso.enable_roles(User, Session)
    #
    # Base.metadata.create_all(engine)
    #
    # policy = """
    # resource(_type: One, "one", ["read"], {boss: {permissions: ["read"]}});
    # resource(_type: Two, "two", ["read"], _roles);
    # """
    # oso.load_str(policy)
    # # TODO: validation
    # # oso.roles.synchronize_data()
    #
    # steve = User(name="steve")
    # one = One(id=one_id)
    #
    # session.add(steve)
    # session.add(one)
    # session.commit()
    #
    # assign_role(steve, one, "boss", session)
    # session.commit()
    # assert oso.is_allowed(steve, "read", one)


def test_role_allows_with_other_rules(init_oso, sample_data):
    oso, session = init_oso
    policy = """
    # Users can read their own data.
    allow(user: User, "read", user);

    resource(_type: Org, "org", actions, roles) if
        actions = ["read"] and
        roles = {
            member: {
                permissions: ["read"]
            }
        };

    allow(_, _, resource) if resource = 1;
    allow(_, _, resource: Boolean) if resource;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    osohq = sample_data["osohq"]
    leina = sample_data["leina"]

    assign_role(leina, osohq, "member", session=session)
    session.commit()

    # This is just to ensure we don't modify the policy above.
    assert oso.is_allowed(leina, "read", osohq)
    assert oso.is_allowed(leina, "read", 1)
    assert not oso.is_allowed(leina, "read", 2)
    assert oso.is_allowed(leina, "read", True)
    assert not oso.is_allowed(leina, "read", False)


# LEGACY TESTS


def test_roles_integration(init_oso, sample_data):
    oso, session = init_oso

    policy = """
    resource(_type: Org, "org", actions, roles) if
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

    resource(_type: Repo, "repo", actions, roles) if
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

    resource(_type: Issue, "issue", actions, {}) if
        actions = [
            "edit"
        ];

    parent(repo: Repo, parent_org) if
        repo.org = parent_org and
        parent_org matches Org;

    parent(issue: Issue, parent_repo) if
        issue.repo = parent_repo and
        parent_repo matches Repo;

    actor_role(actor, role) if
        role in actor.repo_roles or
        role in actor.org_roles;

    allow(actor, action, resource) if
        role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

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

    oso.actor = leina
    oso.checked_permissions = {Repo: "pull"}
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


# Legacy test from sam/polar-roles
def test_legacy_sam_polar_roles(init_oso, sample_data):
    oso, session = init_oso

    policy = """
        resource(_: Org, "org", actions, roles) if
            actions = ["create_repo", "invite"] and
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

        resource(_: Repo, "repo", actions, roles) if
            actions = ["pull", "push"] and
            roles = {
                writer: {
                    permissions: ["push"],
                    implies: ["reader"]
                },
                reader: {
                    permissions: ["pull"]
                }
            };

        parent(repo: Repo, org) if
            org = repo.org and
            org matches Org;

        actor_role(actor, role) if
            role in actor.repo_roles or
            role in actor.org_roles;

        allow(actor, action, resource) if
            role_allow(actor, action, resource);
    """
    oso.load_str(policy)
    oso.validate_config()

    leina = sample_data["leina"]
    steve = sample_data["steve"]
    gabe = sample_data["gabe"]

    osohq = sample_data["osohq"]
    apple = sample_data["apple"]

    oso_repo = sample_data["oso_repo"]
    ios = sample_data["ios"]

    # Things that happen in the app via the management api.
    assign_role(leina, osohq, "owner", session)
    assign_role(steve, osohq, "member", session)
    assign_role(gabe, oso_repo, "writer", session)

    # Test

    # Test Org roles
    # Leina can invite people to osohq because she is an OWNER
    assert oso.is_allowed(leina, "invite", osohq)
    assert not oso.is_allowed(leina, "invite", apple)

    # Steve can create repos in osohq because he is a MEMBER
    assert oso.is_allowed(steve, "create_repo", osohq)

    # Steve can't invite people to osohq because only OWNERs can invite, and he's not an OWNER
    assert not oso.is_allowed(steve, "invite", osohq)

    # Leina can create a repo because she's the OWNER and OWNER implies MEMBER
    assert oso.is_allowed(leina, "create_repo", osohq)

    assert oso.is_allowed(steve, "pull", oso_repo)
    assert not oso.is_allowed(steve, "pull", ios)
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

    # TODO(gj): look at wowhack in sqlalchemy_oso/partial.py
    # # Data filtering test:
    # auth_filter = authorize_model(oso, leina, "push", session, Repo)
    # assert str(auth_filter) == ":param_1 = repositories.organization_id"
    # authorized_repos = session.query(Repo).filter(auth_filter).all()
    # assert len(authorized_repos) == 1
    # assert authorized_repos[0] == oso_repo


@pytest.mark.skipif(not os.environ.get("PERF"), reason="this b slow")
def test_perf_polar(init_oso, sample_data):
    oso, session = init_oso

    # Test many direct roles
    p = """
        resource(_: Repo, "repo", actions, roles) if
        actions = ["read", "write"] and
        roles = {
            reader: {
                permissions: ["read"]
            },
            writer: {
                permissions: ["write"]
            }
        };

        actor_role(actor, role) if
            role in actor.repo_roles or
            role in actor.org_roles;
        """

    # p = """resource(_: Repo, "repo", actions, roles) if
    # actions = ["pull", "push"] and
    # roles = {
    # 	writer: {
    # 	permissions: ["push"],
    # 	implies: ["reader"]
    # 	},
    # 	reader: {
    # 	permissions: ["pull"]
    # 	}
    # };

    # parent(repo: Repo, org) if
    # org = repo.org and org matches Org;
    # """
    oso.load_str(p)
    oso.validate_config()

    leina = sample_data["leina"]
    # steve = sample_data["steve"]
    osohq = sample_data["osohq"]
    # oso_repo = sample_data["oso_repo"]

    # Create 100 repositories
    oso_repos = []
    for i in range(100):
        name = f"oso_repo_{i}"
        repo = Repo(name=name, org=osohq)
        oso_repos.append(repo)
        session.add(repo)

    session.commit()

    n_roles = 100
    for i in range(n_roles):
        assign_role(leina, oso_repos[i], "writer", session)
    session.commit()

    assert len(leina.repo_roles) == n_roles

    number = 10
    time = timeit.timeit(
        lambda: oso.is_allowed(leina, "write", oso_repos[99]), number=number
    )
    print(f"Executed in : {time/number*1000} ms\n Averaged over {number} repetitions.")
