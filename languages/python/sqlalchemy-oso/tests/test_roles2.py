# type: ignore

import pytest
import datetime

from sqlalchemy import create_engine
from sqlalchemy.types import Integer, String, DateTime
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.orm import relationship, sessionmaker
from sqlalchemy.ext.declarative import declarative_base

from sqlalchemy_oso import roles as oso_roles, register_models
from sqlalchemy_oso.roles2 import OsoRoles
from sqlalchemy_oso.session import set_get_session
from oso import Oso, Variable

Base = declarative_base(name="RoleBase")


class Organization(Base):
    __tablename__ = "organizations"

    name = Column(String(), primary_key=True)

    def repr(self):
        return {"name": self.name}


class User(Base):
    __tablename__ = "users"

    user_id = Column(Integer, primary_key=True)
    name = Column(String())

    def repr(self):
        return {"id": self.user_id, "name": self.name}


class Repository(Base):
    __tablename__ = "repositories"

    repo_id = Column(Integer, primary_key=True)
    name = Column(String(256))

    # many-to-one relationship with organizations
    organization_id = Column(Integer, ForeignKey("organizations.name"))
    organization = relationship("Organization", backref="repositories", lazy=True)

    # time info
    created_date = Column(DateTime, default=datetime.datetime.utcnow)
    updated_date = Column(DateTime, default=datetime.datetime.utcnow)

    def repr(self):
        return {"id": self.repo_id, "name": self.name}


class Issue(Base):
    __tablename__ = "issues"

    issue_id = Column(Integer, primary_key=True)
    name = Column(String(256))
    repository_id = Column(Integer, ForeignKey("repositories.repo_id"))
    repository = relationship("Repository", backref="issues", lazy=True)


def test_roles2():
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)
    Session = sessionmaker(bind=engine)
    session = Session()

    steve = User(name="steve")
    leina = User(name="leina")
    osohq = Organization(name="osohq")

    objs = [steve, leina, osohq]

    for obj in objs:
        session.add(obj)
    session.commit()

    oso = Oso()
    roles = OsoRoles(Base)

    register_models(oso, Base)

    ### Basic resource role configuration ###

    # Organizations

    # Define organization permissions
    permission_org_invite = roles.new_permission(resource=Organization, action="invite")
    permission_org_create_repo = roles.new_permission(
        resource=Organization, action="create_repo"
    )

    # Define organization roles
    role_org_owner = roles.new_role(resource=Organization, name="OWNER")
    role_org_member = roles.new_role(resource=Organization, name="MEMBER")

    # Add permissions to organization roles
    roles.add_role_permission(role=role_org_owner, permission=permission_org_invite)
    roles.add_role_permission(
        role=role_org_member, permission=permission_org_create_repo
    )

    # Implied roles for organizations
    roles.add_role_implies(role_org_owner, role_org_member)

    # Repositories

    # Define repo permissions
    permission_repo_push = roles.new_permission(resource=Repository, action="push")
    permission_repo_pull = roles.new_permission(resource=Repository, action="pull")

    # Define repo roles
    role_repo_write = roles.new_role(resource=Repository, name="WRITE")
    role_repo_read = roles.new_role(resource=Repository, name="READ")

    # Add permissions to repo roles
    roles.add_role_permission(role=role_repo_write, permission=permission_repo_push)
    roles.add_role_permission(role=role_repo_read, permission=permission_repo_pull)

    # Implied roles for repositories
    roles.add_role_implies(role_repo_write, role_repo_read)

    ### Relationships + cross-resource implications ###

    # # organizations are the parent of repos
    # roles.new_relationship(
    #     name="repo_org",
    #     child=Repository,
    #     parent=Organization,
    #     parent_selector=lambda child: child.org,
    # )

    # Org "OWNER" role implies repo "WRITE" role for every repo in the org
    roles.add_role_implies(role_org_owner, role_repo_write)
    # Org "MEMBER" role implies repo "READ" role for every repo in the org
    roles.add_role_implies(role_org_member, role_repo_read)

    roles.enable(oso)  # role_allows rule gets added here probably

    oso.load_str(
        """
        allow(actor, action, resource) if
            # if resource is partial, skip and do this at the end
            Roles.role_allows(actor, action, resource);

        role_allows(actor, action, resource) if
            # get the resource tree
            Roles.role_allows(actor, action resource,tree)
            # get all related resources
            # get all roles that have a permission that matches "action:resource_type"
        """
    )

    assert oso.is_allowed(leina, "create_private_repo", osohq)
