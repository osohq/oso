from sqlalchemy.types import Integer, String, DateTime
from sqlalchemy.schema import Table, Column, ForeignKey
from sqlalchemy.ext.declarative import declared_attr
from sqlalchemy.orm import relationship, scoped_session, backref
from sqlalchemy import inspect


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


def get_role_model_for_resource_model(resource_model):
    return inspect(resource_model).relationships.get("roles").argument.class_


def get_user_model_for_resource_model(resource_model):
    role_model = get_role_model_for_resource_model(resource_model)
    return inspect(role_model).relationships.get("users").argument()


# Generic way to get a user's resources and roles for any resource model
def get_user_resources_and_roles(session, user, resource_model):
    """Get a user's roles for a all resources of a single resource type"""
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = type(user)
    resource_roles = (
        session.query(resource_model, role_model)
        .join(role_model)
        .filter(role_model.users.any(user_model.id == user.id))
        .order_by(resource_model.id)
        .order_by(role_model.name)
        .all()
    )
    return resource_roles


def get_group_resources_and_roles(session, group, resource_model):
    """Get a group's roles for a all resources of a single resource type"""
    role_model = get_role_model_for_resource_model(resource_model)
    group_model = type(group)
    resource_roles = (
        session.query(resource_model, role_model)
        .join(role_model)
        .filter(role_model.groups.any(group_model.id == group.id))
        .order_by(resource_model.id)
        .order_by(role_model.name)
        .all()
    )
    return resource_roles


def get_user_roles_for_resource(session, user, resource):
    """Get a user's roles for a single resource record"""
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = type(user)
    roles = (
        session.query(role_model)
        .filter(role_model.users.any(user_model.id == user.id))
        .all()
    )
    return roles


# - Get an organization's users and their roles
def get_resource_users_and_roles(session, resource):
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = get_user_model_for_resource_model(resource_model)
    user_roles = (
        session.query(user_model, role_model)
        .select_from(role_model)
        .join(role_model.users)
        .join(resource_model)
        .filter(resource_model.id == resource.id)
        .order_by(user_model.id)
        .order_by(role_model.name)
        .all()
    )
    return user_roles


# - Get all the users who have a specific role
def get_resource_users_with_role(session, resource, role_name):
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = get_user_model_for_resource_model(resource_model)

    users = (
        session.query(user_model)
        .select_from(role_model)
        .join(role_model.users)
        .join(resource_model)
        .filter(role_model.name == role_name, resource_model.id == resource.id)
        .order_by(user_model.id)
        .all()
    )

    return users


# - Assign a user to an organization with a role
def add_user_role(session, user, resource, role_name):
    # TODO: check input for valid role name
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = type(user)

    # try to get role
    role = (
        session.query(role_model)
        .select_from(resource_model)
        .join(role_model)
        .filter(resource_model.id == resource.id)
        .filter(role_model.name == role_name)
    ).first()

    if role:
        # TODO: check if user already in role
        role.users.append(user)
    else:
        resource_name = resource_model.__name__.lower()
        kwargs = {"name": role_name, resource_name: resource, "users": [user]}

        role = role_model(**kwargs)
        session.add(role)
        session.commit()


# - Delete a user to an organization with a role
def delete_user_role(session, user, resource, role_name=None):
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = type(user)
    resource_name = resource_model.__name__.lower()
    kwargs = {"name": role_name, resource_name: resource, "users": [user]}

    role_query = (
        session.query(role_model)
        .select_from(resource_model)
        .join(role_model)
        .filter(resource_model.id == resource.id)
    )
    if role_name:
        role_query = role_query.filter(role_model.name == role_name)
    else:
        role_query = role_query.filter(role_model.users.any(user_model.id == user.id))

    roles = role_query.all()

    for role in roles:
        try:
            role.users.remove(user)
        except ValueError:
            raise Exception(f"User {user.id} not in role {role.name} for {resource.id}")


# - Change the user's role in an organization
def reassign_user_role(session, user, resource, role_name):
    delete_user_role(session, user, resource)
    add_user_role(session, user, resource, role_name)