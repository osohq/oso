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
        # class Test:
        #     @staticmethod
        #     def hello():
        #         return "world"

        # oso.register_class(Test)
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

        class RolePermission:
            __tablename__ = "role_permission"
            id = Column(Integer, primary_key=True)
            role_id = Column(Integer, ForeignKey("roles.id"))
            permission_id = Column(Integer, ForeignKey("permissions.id"))

        # If you have from_role, you also get to_role
        # add types probably
        class ImpliedRole(sqlalchemy_base):
            __tablename__ = "implied_roles"
            id = Column(Integer, primary_key=True)
            from_role_id = Column(Integer, ForeignKey("roles.id"))
            to_role_id = Column(Integer, ForeignKey("roles.id"))

        class UserRole(sqlalchemy_base):
            __tablename__ = "user_roles"
            id = Column(Integer, primary_key=True)
            user_id =  
            resource: Any
            role_id: int
