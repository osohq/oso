from sqlalchemy import Boolean, Column, Enum, ForeignKey, Integer, String
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship
from sqlalchemy.schema import Table

ModelBase = declarative_base(name="ModelBase")


class Category(ModelBase):
    __tablename__ = "category"

    name = Column(String, primary_key=True)

    tags = relationship("Tag", secondary="category_tags", back_populates="categories")
    users = relationship("User", secondary="category_users")


category_users = Table(
    "category_users",
    ModelBase.metadata,
    Column("user_id", Integer, ForeignKey("users.id")),
    Column("category_name", String, ForeignKey("category.name")),
)


category_tags = Table(
    "category_tags",
    ModelBase.metadata,
    Column("tag_name", String, ForeignKey("tags.name")),
    Column("category_name", String, ForeignKey("category.name")),
)


class Tag(ModelBase):
    __tablename__ = "tags"

    name = Column(String, primary_key=True)
    created_by_id = Column(Integer, ForeignKey("users.id"))
    created_by = relationship("User", foreign_keys=[created_by_id])

    users = relationship("User", secondary="user_tags", back_populates="tags")
    categories = relationship(
        "Category", secondary="category_tags", back_populates="tags"
    )

    # If provided, posts in this tag always have the public access level.
    is_public = Column(Boolean, default=False, nullable=False)


post_tags = Table(
    "post_tags",
    ModelBase.metadata,
    Column("post_id", Integer, ForeignKey("posts.id")),
    Column("tag_id", Integer, ForeignKey("tags.name")),
)

user_tags = Table(
    "user_tags",
    ModelBase.metadata,
    Column("user_id", Integer, ForeignKey("users.id")),
    Column("tag_id", Integer, ForeignKey("tags.name")),
)


post_users = Table(
    "post_users",
    ModelBase.metadata,
    Column("post_id", Integer, ForeignKey("posts.id")),
    Column("user_id", Integer, ForeignKey("users.id")),
)


class Post(ModelBase):
    __tablename__ = "posts"

    id = Column(Integer, primary_key=True)
    contents = Column(String)
    title = Column(String)
    access_level = Column(Enum("public", "private"), nullable=False, default="private")

    created_by_id = Column(Integer, ForeignKey("users.id"))
    created_by = relationship("User", backref="posts")

    users = relationship("User", secondary=post_users)

    needs_moderation = Column(Boolean, nullable=False, default=False)

    tags = relationship("Tag", secondary=post_tags, backref="posts")


class User(ModelBase):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True)
    username = Column(String, nullable=False)

    is_moderator = Column(Boolean, nullable=False, default=False)
    is_banned = Column(Boolean, nullable=False, default=False)

    # Single tag
    tag_name = Column(Integer, ForeignKey("tags.name"))
    tag = relationship("Tag", foreign_keys=[tag_name])

    # Many tags
    tags = relationship("Tag", secondary=user_tags, back_populates="users")
