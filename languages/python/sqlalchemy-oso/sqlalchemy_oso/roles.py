from sqlalchemy.types import Integer, String, DateTime
from sqlalchemy.schema import Table, Column, ForeignKey
from sqlalchemy.ext.declarative import declared_attr
from sqlalchemy.orm import relationship, scoped_session, backref


def resource_role_class(
    declarative_base, user_model, resource_model, roles, group_model=None
):
    # many-to-many relationship with users
    user_join_table = Table(
        f"{resource_model.__name__.lower()}_roles_users",
        declarative_base.metadata,
        Column(
            f"{resource_model.__name__.lower()}_role_id",
            Integer,
            ForeignKey(f"{resource_model.__name__.lower()}_roles.id"),
            primary_key=True,
        ),
        Column(
            "user_id",
            Integer,
            ForeignKey(f"{user_model.__tablename__}.id"),
            primary_key=True,
        ),
    )

    class ResourceRoleMixin:
        choices = roles

        __tablename__ = f"{resource_model.__name__.lower()}_roles"
        id = Column(Integer, primary_key=True)
        name = Column(String())

        # many-to-many relationship with users
        @declared_attr
        def users(cls):
            return relationship(
                f"{user_model.__name__}",
                secondary=user_join_table,
                lazy="subquery",
                backref=backref(f"{resource_model.__name__.lower()}_roles", lazy=True),
            )

    @declared_attr
    def resource_id(cls):
        table_name = resource_model.__tablename__
        return Column(Integer, ForeignKey(f"{table_name}.id"))

    @declared_attr
    def resource(cls):
        return relationship(resource_model.__name__, backref="roles", lazy=True)

    setattr(ResourceRoleMixin, f"{resource_model.__name__.lower()}_id", resource_id)
    setattr(ResourceRoleMixin, resource_model.__name__.lower(), resource)

    if group_model:
        group_join_table = Table(
            f"{resource_model.__name__.lower()}_roles_groups",
            declarative_base.metadata,
            Column(
                f"{resource_model.__name__.lower()}_role_id",
                Integer,
                ForeignKey(f"{resource_model.__name__.lower()}_roles.id"),
                primary_key=True,
            ),
            Column(
                "group_id",
                Integer,
                ForeignKey(f"{group_model.__tablename__}.id"),
                primary_key=True,
            ),
        )

        @declared_attr
        def groups(cls):
            return relationship(
                f"{group_model.__name__}",
                secondary=group_join_table,
                lazy="subquery",
                backref=backref(f"{group_model.__name__.lower()}_roles", lazy=True),
            )

        setattr(ResourceRoleMixin, "groups", groups)

    return ResourceRoleMixin
