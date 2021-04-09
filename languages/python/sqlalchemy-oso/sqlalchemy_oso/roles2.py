from typing import Any, List

from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.ext.declarative import declared_attr
from sqlalchemy.orm import relationship, validates, class_mapper
from sqlalchemy.orm.util import object_mapper
from sqlalchemy.orm.exc import UnmappedInstanceError, UnmappedClassError
from sqlalchemy import inspect, UniqueConstraint
from sqlalchemy.exc import IntegrityError


class OsoRoles:
    def __init__(self, sqlalchemy_base):
        self.base = sqlalchemy_base

    def enable(self, oso, sqlalchemy_base, user_model):
        # TODO: check that user_model is valid

        class Permission(sqlalchemy_base):
            __tablename__ = "permissions"
            id = Column(Integer, primary_key=True)
            resource_type = Column(String)
            action = Column(String)

        class Role(sqlalchemy_base):
            __tablename__ = "roles"
            id = Column(Integer, primary_key=True)
            resource_type = Column(String)
            name = Column(String)

        class RolePermission(sqlalchemy_base):
            __tablename__ = "role_permission"
            id = Column(Integer, primary_key=True)
            role_id = Column(Integer, ForeignKey("roles.id"))
            permission_id = Column(Integer, ForeignKey("permissions.id"))

        # TODO: how do we store relationships?
        class ParentRelationship(sqlalchemy_base):
            id = Column(Integer, primary_key=True)
            name = Column(String)
            child_type = Column(String)
            parent_type = Column(String)
            parent_field = Column(String)

        # If you have from_role, you also get to_role
        # add types probably
        class ImpliedRole(sqlalchemy_base):
            __tablename__ = "implied_roles"
            id = Column(Integer, primary_key=True)
            from_role_id = Column(Integer, ForeignKey("roles.id"))
            to_role_id = Column(Integer, ForeignKey("roles.id"))

        user_pk_type = inspect(user_model).primary_key[0].type
        user_pk_name = inspect(user_model).primary_key[0].name
        user_table_name = user_model.__tablename__

        class UserRole(sqlalchemy_base):
            __tablename__ = "user_roles"
            id = Column(Integer, primary_key=True)
            user_id = Column(
                user_pk_type, ForeignKey(f"{user_table_name}.{user_pk_name}")
            )
            # TODO: this will only work when the resource PK is a single integer
            # We may need to have a new `UserRole` table for every role type...
            resource_type = Column(String)
            resource_id = Column(Integer)
            role_id = Column(Integer)

        def configure(self, resource_types, relationships, permissions, roles):
            pass
            # TODO: input validation
            # self.resource_types = resource_types
            # for name, field in relationships:
            #     new

