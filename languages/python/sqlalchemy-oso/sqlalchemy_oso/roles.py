from typing import Any, List

from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Table, Column, ForeignKey
from sqlalchemy.ext.declarative import declared_attr
from sqlalchemy.orm import relationship, backref, validates
from sqlalchemy.event import listen
from sqlalchemy import inspect, UniqueConstraint
from .session import _OsoSession

ROLE_CLASSES: List[Any] = []


def resource_role_class(
    declarative_base, user_model, resource_model, role_choices, mutually_exclusive=True
):
    """Create a :ref:`resource-specific role<resource-specific-roles>` Mixin
    for SQLAlchemy models. The role mixin is an `Association
    Object<https://docs.sqlalchemy.org/en/13/orm/basic_relationships.html#association-object>`_
    between the ``user_model`` and the ``resource_model``.


    :param declarative_base: The SQLAlchemy declarative base model that \
    the role model and all related models are mapped to.

    :param user_model: The SQLAlchemy model representing users that the \
    resource-specific roles can be assigned to. The generated Role mixin will \
    have a many-to-one (Foreign Key) relationship with this user model. \
    A many-to-many relationship to ``resource_model`` is added to ``user_model``; \
    the relationship is named following the convention: ``resource_model.__name__.lower() + "s"``.

    :param resource_model: The SQLAlchemy model representing resources that \
    the generated Role mixin will be scoped to. The Role mixin will \
    have a many-to-one (ForeignKey) relationship with this resource model. \
    A many-to-many relationship to ``user_model`` is added to ``resource_model``; \
    the relationship is named ``users``.

    :param roles: An order-independent list of the built-in roles for this resource-specific role type.
    :type roles: List[str]

        .. code-block:: python

            class Team(Base):
                __tablename__ = "teams"

                id = Column(Integer, primary_key=True)
                name = Column(String(256))

    :param mutually_exclusive: Boolean flag that sets whether or not users \
    can have more than one role for a given resource. Defaults to ``True``.
    :type roles: bool

    :return: the ResourceRole mixin, which must then be mixed into a SQLAlchemy model for the role. E.g.,

        .. code-block:: python

            OrganizationRoleMixin = oso_roles.resource_role_class(
                Base, User, Organization, ["OWNER", "MEMBER", "BILLING"]
            )

            class OrganizationRole(Base, OrganizationRoleMixin):
                pass


    """
    global ROLE_CLASSES
    ROLE_CLASSES.append(
        {
            "user_model": user_model,
            "resource_model": resource_model,
        }
    )

    tablename = f"{resource_model.__name__.lower()}_roles"
    if mutually_exclusive:
        unique_constraint = UniqueConstraint(
            f"{resource_model.__name__.lower()}_id", "user_id"
        )
    else:
        unique_constraint = UniqueConstraint(
            f"{resource_model.__name__.lower()}_id", "name", "user_id"
        )

    class ResourceRoleMixin:
        choices = role_choices

        __tablename__ = tablename
        id = Column(Integer, primary_key=True)
        name = Column(String())
        __table_args__ = (unique_constraint,)

        @validates("name")
        def validate_name(self, key, name):
            if name not in self.choices:
                raise ValueError(
                    f"{name} Is not a valid choice for {self.__class__.__name__}"
                )
            return name

        @declared_attr
        def user_id(cls):
            type = inspect(user_model).primary_key[0].type
            name = inspect(user_model).primary_key[0].name
            table_name = user_model.__tablename__
            return Column(type, ForeignKey(f"{table_name}.{name}"))

        @declared_attr
        def user(cls):
            return relationship(user_model.__name__, backref=tablename, lazy=True)

    @declared_attr
    def resource_id(cls):
        type = inspect(resource_model).primary_key[0].type
        name = inspect(resource_model).primary_key[0].name
        table_name = resource_model.__tablename__
        return Column(type, ForeignKey(f"{table_name}.{name}"))

    @declared_attr
    def resource(cls):
        return relationship(resource_model.__name__, backref="roles", lazy=True)

    setattr(ResourceRoleMixin, f"{resource_model.__name__.lower()}_id", resource_id)
    setattr(ResourceRoleMixin, resource_model.__name__.lower(), resource)

    # Add the relationship between the user_model and the resource_model
    resources = relationship(
        resource_model.__name__,
        secondary=tablename,
        lazy=True,
        viewonly=True,
        backref="users",
        sync_backref=False,
    )
    # @Q: Do we try to pluralize this name correctly?
    setattr(user_model, resource_model.__name__.lower() + "s", resources)

    return ResourceRoleMixin


def enable_roles(oso):
    # TODO: ensure this docstring is still accurate
    """Enable the SQLAlchemy Role-Based Access Control base policy. This method activates the following polar rules:

    ``role_allow(role, action, resource)``:
        Allows actors that have the role ``role`` to take ``action`` on
        ``resource``. ``role`` is a SQLAlchemy role model generated by
        :py:meth:`sqlalchemy_oso.roles.resource_role_class`. ``resource``
        is a SQLAlchemy model to which the ``role`` applies. Roles apply
        to the resources they are scoped to, For example,
        ``OrganizationRole`` roles apply to ``Organization`` resources.
        Roles may also apply to resources as specified by
        ``resource_role_applies_to`` Polar rules. E.g.,

        .. code-block:: polar

            role_allow(role: OrganizationRole{name: "MEMBER"}, "READ", org: Organization);


    ``resource_role_applies_to(child_resource, parent_resource)``:
        Permits roles that control access to `parent_resource` apply to
        `child_resource` as well. `parent_resource` must be a resource
        that has a resource role class associated with it (see
        :py:meth:`sqlalchemy_oso.roles.resource_role_class`). E.g.,

        .. code-block:: polar

            ### An organization's roles apply to its child repositories
            resource_role_applies_to(repo: Repository, parent_org) if
                parent_org = repo.organization;

        The above rule makes it possible to write `role_allow` rules
        between `OrganizationRole` and `Repository`. E.g.,

        .. code-block:: polar

            role_allow(role: OrganizationRole{name: "MEMBER"}, "READ", repo: Repository);

    ``[resource_name]_role_order(["ROLE_NAME_1", "ROLE_NAME_2",...])``:
        Specifies a hierarchical role order for built-in
        resource-specific roles defined with
        :py:meth:`sqlalchemy_oso.roles.resource_role_class` The rule name
        is the lower-cased resource model name followed by
        ``_role_order``. The only parameter is a list of role names in
        hierarchical order. Roles to the left will inherit the
        permissions of roles to the right. This is useful if any role
        should inherit all the permissions of another role. It is not
        required for all built-in roles to be specified in the list. E.g.,

        .. code-block:: polar

            repository_role_order(["ADMIN", "MAINTAIN", "WRITE", "TRIAGE", "READ"]);

        Is the equivalent of writing:

        .. code-block:: polar

            role_allow(role: RepositoryRole{name: "ADMIN"}, _action, _resource) if
                role_allow(new RepositoryRole{name: "MAINTAIN"}, _action, _resource);

            role_allow(role: RepositoryRole{name: "MAINTAIN"}, _action, _resource) if
                role_allow(new RepositoryRole{name: "WRITE"}, _action, _resource);

        ...and so on.


    :param oso: The Oso instance used to evaluate the policy.
    :type oso: Oso
    """

    if not _OsoSession.set:
        raise Exception(
            "Sqlalchemy roles requires the sqlalchemy OsoSession. Please call session.set_get_session before enable_roles."
        )

    global ROLE_CLASSES

    policy = """
    # RBAC BASE POLICY

    ## Top-level RBAC allow rule

    ### The association between the resource roles and the requested resource is outsourced from the rbac_allow
    allow(user, action, resource) if
        resource_role_applies_to(resource, role_resource) and
        user_in_role(user, role, role_resource) and
        role_allow(role, action, resource);

    # RESOURCE-ROLE RELATIONSHIPS

    ## These rules allow roles to apply to resources other than those that they are scoped to.
    ## The most common example of this is nested resources, e.g. Repository roles should apply to the Issues
    ## nested in that repository.

    ### A resource's roles applies to itself
    resource_role_applies_to(role_resource, role_resource);

    # ROLE-ROLE RELATIONSHIPS

    ## Role Hierarchies

    ### Grant a role permissions that it inherits from a more junior role
    role_allow(role, action, resource) if
        inherits_role(role, inherited_role) and
        role_allow(inherited_role, action, resource);

    ### Helper to determine relative order or roles in a list
    inherits_role_helper(role, inherited_role, role_order) if
        ([first, *rest] = role_order and
        role = first and
        inherited_role in rest) or
        ([first, *rest] = role_order and
        inherits_role_helper(role, inherited_role, rest));
    """

    for role_model in ROLE_CLASSES:
        UserModel = role_model["user_model"]
        User = UserModel.__name__
        ResourceModel = role_model["resource_model"]
        Resource = ResourceModel.__name__
        Role = get_role_model_for_resource_model(ResourceModel).__name__

        policy += f"""
        user_in_role(user: {User}, role, resource: {Resource}) if
            session = OsoSession.get() and
            role in session.query({Role}).filter_by(user: user, {Resource.lower()}: resource).all();

        inherits_role(role: {Role}, inherited_role) if
            {Resource.lower()}_role_order(role_order) and
            inherits_role_helper(role.name, inherited_role_name, role_order) and
            inherited_role = new {Role}(name: inherited_role_name, {Resource.lower()}: role.{Resource.lower()});
        """
    oso.load_str(policy)


# ROLE HELPERS


def get_role_model_for_resource_model(resource_model):
    try:
        return (
            inspect(resource_model, raiseerr=True)
            .relationships.get("roles")
            .argument.class_
        )
    except AttributeError:
        raise TypeError(f"Expected a model; received: {resource_model}")


def get_user_model_for_resource_model(resource_model):
    try:
        return inspect(resource_model).relationships.get("users").argument.class_
    except AttributeError:
        raise TypeError(f"Expected a model; received: {resource_model}")


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
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = type(user)

    roles = (
        session.query(role_model)
        .join(resource_model)
        .filter(role_model.user == user)
        .order_by(resource_model.id)
        .order_by(role_model.name)
    )

    if resource_id:
        roles = roles.filter(resource_model.id == resource_id)
    return roles.all()


def get_resource_roles(session, resource):
    """Get all of the roles for a specific resource. E.g.,
    get all the roles in Organization 1. Each role has a single user
    associated with it, which can be accessed by calling ``role.user``.

    :param session: SQLAlchemy session
    :type session: sqlalchemy.orm.session.Session

    :param resource: the resource record (python object) for which to get \
    the users and roles

    :return: list of the user's roles
    :return: List of roles associated with the ``resource``

    """
    return resource.roles


# - Get all the users who have a specific role
def get_resource_users_by_role(session, resource, role_name):
    """Get all of the users that have a specific role for a specific
    resource. E.g., get all the users in Organization 1 that have the "OWNER"
    role.

    :param session: SQLAlchemy session
    :type session: sqlalchemy.orm.session.Session

    :param resource: the resource record (python object) for which to get \
    the users

    :param role_name: the name of the role to get users for
    :type role_name: str

    :return: List of users that have the ``role_name`` role for \
    ``resource``

    """
    # TODO: would it be helpful to aggregate the roles by name if `role_name`
    # is None? E.g. return a dict of {role_name: [users]}?
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = get_user_model_for_resource_model(resource_model)

    users = (
        session.query(user_model)
        .join(role_model)
        .filter_by(repository=resource, name=role_name)
        .order_by(user_model.id)
        .all()
    )

    return users


# - Assign a user to an organization with a role
def add_user_role(session, user, resource, role_name):
    """Add a user to a role for a specific resource.

    :param session: SQLAlchemy session
    :type session: sqlalchemy.orm.session.Session

    :param user: user record (python object) to assign the role to

    :param role_name: the name of the role to assign to the user
    :type role_name: str
    """
    # get models
    resource_model = type(resource)
    role_model = get_role_model_for_resource_model(resource_model)

    # create and save role
    resource_name = resource_model.__name__.lower()
    kwargs = {"name": role_name, resource_name: resource, "user": user}
    new_role = role_model(**kwargs)
    session.add(new_role)
    session.commit()


# - Delete a user to an organization with a role
def delete_user_role(session, user, resource, role_name=None):
    """Remove a user from a role for a specific resource.

    :param session: SQLAlchemy session
    :type session: sqlalchemy.orm.session.Session

    :param user: user record (python object) to remove the role from

    :param role_name: the name of the role to remove from the user. If not \
    provided, the function will remove all roles the user has for \
    ``resource``.
    :type role_name: str
    """
    resource_model = type(resource)
    resource_name = resource_model.__name__.lower()
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = type(user)

    filter_kwargs = {"user": user, resource_name: resource}
    if role_name:
        filter_kwargs["name"] = role_name
    roles = session.query(role_model).filter_by(**filter_kwargs)

    roles.delete()
    session.commit()


# - Change the user's role in an organization
def reassign_user_role(session, user, resource, role_name):
    """Remove all existing roles that a user has for a specific resource, and
    reassign the user to a new role. If the user does not have any roles for
    the given resource, the behavior is the same as
    :py:meth:`sqlalchemy_oso.roles.add_user_role`.

    :param session: SQLAlchemy session
    :type session: sqlalchemy.orm.session.Session

    :param user: user record (python object) whose role should be reassigned

    :param role_name: the name of the new role to assign to the user
    :type role_name: str
    """
    resource_model = type(resource)
    resource_name = resource_model.__name__.lower()
    role_model = get_role_model_for_resource_model(resource_model)
    user_model = type(user)

    filter_kwargs = {"user": user, resource_name: resource}

    session.query(role_model).filter_by(**filter_kwargs).update({"name": role_name})
    session.commit()
