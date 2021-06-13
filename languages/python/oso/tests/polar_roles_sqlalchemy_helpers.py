# Roles 2
from typing import Any, List

from oso import OsoError

from sqlalchemy import inspect, UniqueConstraint
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.declarative import declared_attr
from sqlalchemy.orm import class_mapper, relationship, validates, synonym
from sqlalchemy.orm.exc import UnmappedClassError, UnmappedInstanceError
from sqlalchemy.orm.util import object_mapper
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.types import Integer, String

# Global list to keep track of role classes as they are created, used to
# generate RBAC base policy in Polar
ROLE_CLASSES: List[Any] = []


def assign_role(user, resource, role_name, session, reassign=True):
    assert session is not None
    pk_name, _ = get_pk(type(resource))
    existing_roles = get_user_roles(
        session, user, type(resource), getattr(resource, pk_name)
    )
    assert len(existing_roles) < 2
    if len(existing_roles) == 1:
        if reassign:
            existing_roles[0].name = role_name
        else:
            raise OsoError(
                f"""User {user} already has a role for this resource.
                To reassign, call with `reassign=True`."""
            )
    else:
        return add_user_role(session, user, resource, role_name, commit=True)


def remove_role(user, resource, role_name, session):
    pk_name, _ = get_pk(type(resource))
    existing_roles = get_user_roles(
        session, user, type(resource), getattr(resource, pk_name)
    )
    assert len(existing_roles) < 2
    if len(existing_roles) == 1:
        assert existing_roles[0].name == role_name
        session.delete(existing_roles[0])
        session.flush()
        return True
    else:
        return False


def resource_role_class(user_model, resource_model, role_choices):
    """Create a resource-specific role Mixin
    for SQLAlchemy models. The role mixin is an
    `Association Object <https://docs.sqlalchemy.org/en/13/orm/basic_relationships.html#association-object>`_
    between the ``user_model`` and the ``resource_model``.

    :param user_model: The SQLAlchemy model representing users that the \
    resource-specific roles can be assigned to. The generated Role mixin will \
    have a many-to-one (Foreign Key) relationship with this user model. \
    A many-to-many relationship to ``resource_model`` is added to ``user_model``; \
    the relationship is named following the convention: ``resource_model.__name__.lower() + "s"``.

    :param resource_model: The SQLAlchemy model representing resources that \
    the generated Role mixin will be scoped to. The Role mixin will \
    have a many-to-one (ForeignKey) relationship with this resource model. \
    A many-to-many relationship to ``user_model`` is added to ``resource_model``; \
    the relationship is named ``users``. \
    NOTE: only one role model can be created per resource model. Attempting to call \
    ``resource_role_class()`` more than once for the same resource model will result in \
    a ``ValueError``.

    :param roles: An order-independent list of the built-in roles for this resource-specific role type.
    :type roles: List[str]

    :type roles: bool

    :return: the ResourceRole mixin, which must then be mixed into a SQLAlchemy model for the role. E.g.,

        .. code-block:: python

            OrganizationRoleMixin = oso_roles.resource_role_class(
                User, Organization, ["OWNER", "MEMBER", "BILLING"]
            )

            class OrganizationRole(Base, OrganizationRoleMixin):
                pass


    """

    global ROLE_CLASSES
    if resource_model in [role.get("resource_model") for role in ROLE_CLASSES]:
        raise ValueError(
            f"Cannot create two Role classes for the same `resource_model`: {resource_model.__name__}"
        )

    ROLE_CLASSES.append(
        {
            "user_model": user_model,
            "resource_model": resource_model,
        }
    )

    resource_name = _get_resource_name_lower(resource_model)
    tablename = f"{resource_name}_roles"
    unique_constraint = UniqueConstraint(f"{resource_name}_id", "user_id")

    class ResourceRoleMixin:
        choices = role_choices

        __tablename__ = tablename
        id = Column(Integer, primary_key=True)
        name = Column(String())
        __table_args__ = (unique_constraint,)

        @validates("name")
        def validate_name(self, _, name):
            if name not in self.choices:
                raise ValueError(
                    f"{name} Is not a valid choice for {self.__class__.__name__}"
                )
            return name

        @declared_attr
        def user_id(cls):
            name, type = get_pk(user_model)
            table_name = user_model.__tablename__
            return Column(type, ForeignKey(f"{table_name}.{name}"))

        @declared_attr
        def user(cls):
            return relationship(user_model.__name__, backref=tablename)

        def __repr__(self):
            return ""

    @declared_attr
    def named_resource_id(cls):
        name, type = get_pk(resource_model)
        table_name = resource_model.__tablename__
        return Column(type, ForeignKey(f"{table_name}.{name}"))

    @declared_attr
    def named_resource(cls):
        return relationship(resource_model.__name__, backref="roles")

    @declared_attr
    def resource(cls):
        return synonym(resource_name)

    setattr(ResourceRoleMixin, f"{resource_name}_id", named_resource_id)
    setattr(ResourceRoleMixin, resource_name, named_resource)
    setattr(ResourceRoleMixin, "resource", resource)

    # Add the relationship between the user_model and the resource_model
    resources = relationship(
        resource_model.__name__,
        secondary=tablename,
        viewonly=True,
        backref="users",
        sync_backref=False,
    )
    # @Q: Do we try to pluralize this name correctly?
    setattr(user_model, resource_name + "s", resources)

    return ResourceRoleMixin


# ROLE HELPERS


def _get_resource_name_lower(resource_model):
    return resource_model.__name__.lower()


def _check_valid_instance(*args, raise_error=True):
    for instance in args:
        valid = True
        try:
            object_mapper(instance)
        except UnmappedInstanceError:
            valid = False

        if raise_error and not valid:
            raise TypeError(f"Expected a mapped object instance; received: {instance}")


def _check_valid_model(*args, raise_error=True):
    for model in args:
        valid = True
        try:
            class_mapper(model)
        except UnmappedClassError:
            valid = False

        if raise_error and not valid:
            raise TypeError(f"Expected a model (mapped class); received: {model}")


def get_pk(model):
    pks = inspect(model).primary_key
    assert (
        len(pks) == 1
    ), "sqlalchemy.roles2 only supports resources with 1 primary key field."
    type = pks[0].type
    name = pks[0].name
    return (name, type)


def get_role_model_for_resource_model(resource_model):
    _check_valid_model(resource_model)
    return (
        inspect(resource_model, raiseerr=True)
        .relationships.get("roles")
        .argument.class_
    )


def get_user_roles(session, user, resource_model, resource_id=None):
    """Get a user's roles for all resources of a single resource type.
    E.g., get all of a user's repositories and their role for each
    repository.
    Or optionally, all roles scoped to a specific resource_id.
    :param session: SQLAlchemy session
    :type session: sqlalchemy.orm.session.Session
    :param user: user record (python object) of the SQLAlchemy user model \
    associated with roles scoped to the supplied ``resource_model``
    :param resource_id: (optional) the resource id for which to get the user's roles.
    :return: list of the user's roles
    """
    _check_valid_instance(user)
    _check_valid_model(resource_model)
    role_model = get_role_model_for_resource_model(resource_model)
    resource_pk, _ = get_pk(resource_model)

    roles = (
        session.query(role_model)
        .join(resource_model)
        .filter(role_model.user == user)
        .order_by(getattr(resource_model, resource_pk))
        .order_by(role_model.name)
    )

    if resource_id:
        roles = roles.filter(getattr(resource_model, resource_pk) == resource_id)
    return roles.all()


# - Assign a user to an organization with a role
def add_user_role(session, user, resource, role_name, commit=False):
    """Add a user to a role for a specific resource.
    :param session: SQLAlchemy session
    :type session: sqlalchemy.orm.session.Session
    :param user: user record (python object) to assign the role to
    :param role_name: the name of the role to assign to the user
    :type role_name: str
    :param commit: flag to specify whether or not session should be committed after adding role; defaults to ``False``
    :type commit: boolean
    """
    _check_valid_instance(user, resource)
    # get models
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)

    # create and save role
    resource_name = _get_resource_name_lower(resource_model)
    kwargs = {"name": role_name, resource_name: resource, "user": user}
    new_role = role_model(**kwargs)
    session.add(new_role)
    if commit:
        try:
            session.commit()
        except IntegrityError:
            session.rollback()
            raise Exception(
                f"""Cannot assign user {user} to role {role_name} for
                {resource_name} either because the assignment already exists."""
            )
