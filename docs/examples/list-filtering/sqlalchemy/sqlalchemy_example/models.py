from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship

from sqlalchemy import Column, String, Integer, Boolean, ForeignKey, Enum, Table

Model = declarative_base(name="Model")

class Post(Model):
    __tablename__ = "posts"

    id = Column(Integer, primary_key=True)

    contents = Column(String)
    access_level = Column(Enum("public", "private"), nullable=False)

    created_by_id = Column(Integer, ForeignKey("users.id"))
    created_by = relationship("User")

"""Represent a management relationship between users.  A record in this table
indicates that ``user_id``'s account can be managed by the user with ``manager_id``.
"""
user_manages = Table(
    "user_manages",
    Model.metadata,
    Column("managed_user_id", Integer, ForeignKey("users.id")),
    Column("manager_user_id", Integer, ForeignKey("users.id"))
)

class User(Model):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True)
    username = Column(String, nullable=False)

    is_admin = Column(Boolean, nullable=False, default=False)

    manages = relationship("User",
        secondary="user_manages",
        primaryjoin=(id == user_manages.c.manager_user_id),
        secondaryjoin=(id == user_manages.c.managed_user_id),
        backref="managed_by"
    )

