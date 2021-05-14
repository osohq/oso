# Roles 2
from typing import Any, List, Set, Dict
from dataclasses import dataclass

from oso import Variable

import sqlalchemy
from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy import inspect
from sqlalchemy.orm import class_mapper
from sqlalchemy.orm.exc import UnmappedClassError
from sqlalchemy import sql

from oso import OsoError


def isa_type(arg):
    assert arg.operator == "Isa"
    assert len(arg.args) == 2
    assert arg.args[0] == Variable("_this")
    pattern = arg.args[1]
    type = pattern.tag
    return type


def get_pk(model):
    pks = inspect(model).primary_key
    assert (
        len(pks) == 1
    ), "sqlalchemy.roles2 only supports resources with 1 primary key field."
    type = pks[0].type
    name = pks[0].name
    return (name, type)


def role_allow_query(
    id_query, type_query, child_types, resource_id_field, has_relationships
):
    child_types = ",".join([f"'{ct}'" for ct in child_types])

    if not has_relationships:
        recur = ""
    else:
        recur = f"""
        union
        select
            {id_query},
            {type_query}
        from resources
        where type in ({child_types})"""
    query = f"""
        -- get all the relevant resources by walking the parent tree
        with resources as (
            with recursive resources (id, type) as (
                select
                    {resource_id_field} as id,
                    :resource_type as type
                {recur}
            ) select * from resources
        ), allow_permission as (
            -- Find the permission
            select
                p.id
            from permissions p
            where p.resource_type = :resource_type and p.name = :action
        ), permission_roles as (
            -- find roles with the permission
            select
                rp.role
            from role_permissions rp
            join allow_permission ap
            on rp.permission_id = ap.id
        ), relevant_roles as (
            -- recursively find all roles that have the permission or
            -- imply a role that has the permission
            with recursive relevant_roles (role) as (
                select
                    role
                from permission_roles
                union
                select
                    ri.from_role
                from role_implications ri
                join relevant_roles rr
                on ri.to_role = rr.role
            ) select * from relevant_roles
        ), user_in_role as (
            -- check if the user has any of those roles on any of the relevant resources
            select
                ur.resource_type,
                ur.resource_id,
                ur.role
            from user_roles ur
            join relevant_roles rr
            on rr.role = ur.role
            join resources r
            on r.type = ur.resource_type and cast(r.id as text) = ur.resource_id
            where ur.user_id = :user_id
        ) select * from user_in_role
    """
    return query


def user_in_role_query(
    id_query, type_query, child_types, resource_id_field, has_relationships
):
    child_types = ",".join([f"'{ct}'" for ct in child_types])

    if not has_relationships:
        recur = ""
    else:
        recur = f"""
        union
        select
            {id_query},
            {type_query}
        from resources
        where type in ({child_types})"""
    query = f"""
        -- get all the relevant resources by walking the parent tree
        with resources as (
            with recursive resources (id, type) as (
                select
                    {resource_id_field} as id,
                    :resource_type as type
                {recur}
            ) select * from resources
        ), role as (
            -- find roles with the permission
            select
                r.name
            from roles r
            where r.name = :role
        ), relevant_roles as (
            -- recursively find all roles that have the permission or
            -- imply a role that has the permission
            with recursive relevant_roles (role) as (
                select
                    name
                from role
                union
                select
                    ri.from_role
                from role_implications ri
                join relevant_roles rr
                on ri.to_role = rr.role
            ) select * from relevant_roles
        ), user_in_role as (
            -- check if the user has any of those roles on any of the relevant resources
            select
                ur.resource_type,
                ur.resource_id,
                ur.role
            from user_roles ur
            join relevant_roles rr
            on rr.role = ur.role
            join resources r
            on r.type = ur.resource_type and cast(r.id as text) = ur.resource_id
            where ur.user_id = :user_id
        ) select * from user_in_role
    """
    return query


# Python representation of the configuration data.
# Currently this data is read from a polar file that has
# resource and parent rule definitions.
@dataclass
class Relationship:
    child_python_class: Any
    child_type: str
    child_table: str
    child_join_column: str
    parent_python_class: Any
    parent_type: str
    parent_table: str
    parent_join_column: str


@dataclass
class Permission:
    python_class: Any
    type: str
    name: str


@dataclass
class Role:
    name: str
    type: str
    python_class: Any
    permissions: List[Permission]
    implied_roles: List[str]


@dataclass
class Resource:
    name: str
    type: str
    python_class: Any
    actions: Set[str]
    roles: Set[str]


@dataclass
class Config:
    resources: Dict[str, Resource]
    permissions: List[Permission]
    roles: Dict[str, Role]
    relationships: List[Relationship]


def parse_permission(permission, python_class, config):
    """Parse a permission string, check if it's valid and return a Permission"""
    if ":" in permission:
        resource_name, action = permission.split(":", 1)
        if resource_name not in config.resources:
            raise OsoError("Invalid permission namespace.")
        permission_python_class = config.resources[resource_name].python_class
    else:
        action = permission
        permission_python_class = python_class
    perm = Permission(
        name=action,
        type=permission_python_class.__name__,
        python_class=permission_python_class,
    )
    if perm not in config.permissions:
        raise OsoError(
            f"Permission {perm.name} doesn't exist for resource {perm.type}."
        )
    return perm


def read_config(oso):
    """
    Queries the polar policy for the configuration
    """
    config = Config(resources={}, permissions=[], roles={}, relationships=[])

    # Register relationships
    role_relationships = oso.query_rule(
        "parent",
        Variable("resource"),
        Variable("parent_resource"),
        accept_expression=True,
    )

    # Currently there is only one valid relationship, a parent.
    # There is also only one way you can write it as a rule in polar.
    # parent(child_resource, parent_resource) if
    #     child.parent = parent_resource;
    #
    # @TODO: Support other forms of this rule, eg
    # parent(child_resource, parent_resource) if
    #     child.parent_id = parent_resource.id;
    for result in role_relationships:
        try:
            constraints = result["bindings"]["resource"]
            assert len(constraints.args) == 2
            type_check = constraints.args[0]
            child_type = isa_type(type_check)
            get_parent = constraints.args[1]
            assert get_parent.operator == "Isa"
            assert len(get_parent.args) == 2
            getter = get_parent.args[0]
            assert getter.operator == "Dot"
            assert len(getter.args) == 2
            assert getter.args[0] == Variable("_this")
            child_attr = getter.args[1]
            pattern = get_parent.args[1]
            parent_type = pattern.tag

            child_python_class = oso.host.classes[child_type]
            child_table = child_python_class.__tablename__
            parent_python_class = oso.host.classes[parent_type]
            parent_table = parent_python_class.__tablename__

            # the rule has the form
            # `child.child_attr = parent`
            # child_attr is assumed to be a sqlalchemy relationship field
            # so we inspect the model to get the actual sql fields to join on
            child_relationships = inspect(child_python_class).relationships
            if child_attr not in child_relationships:
                raise OsoError(
                    f"Invalid Relationship: {child_attr} "
                    + "is not a sqlalchemy relationship field."
                )
            rel = child_relationships[child_attr]
            parent_join_column = list(rel.remote_side)[0].name
            child_join_column = list(rel.local_columns)[0].name

            relationship = Relationship(
                child_python_class=child_python_class,
                child_type=child_type,
                child_table=child_table,
                child_join_column=child_join_column,
                parent_python_class=parent_python_class,
                parent_type=parent_type,
                parent_table=parent_table,
                parent_join_column=parent_join_column,
            )

            config.relationships.append(relationship)
        except AssertionError:
            raise OsoError("Invalid relationship.")

    # Register resources / permissions / roles and implications
    # Based on the role_resource definitions
    # These are rules that look like this.
    # resource(_type: Repository, "repo", actions, roles) if
    #     actions = [
    #         "push",
    #         "pull"
    #     ] and
    #     roles = {
    #         repo_write: {
    #             perms: ["push", "issue:edit"],
    #             implies: ["repo_read"]
    #         },
    #         repo_read: {
    #             perms: ["pull"]
    #         }
    #     };
    # The first argument lets us use a specializer to say what the python class is.
    # The second arg is a name which is used for permission namespacing.
    # The third arg should be bound to a list of actions defined for the resource.
    # The fouth arg should be bound to a map from role name to role definition.
    #   Each role definitions has two fields,
    #     perms which says which permissions the role has
    #     and implies which says which other roles are implied by having this one.
    role_resources = oso.query_rule(
        "resource",
        Variable("resource"),
        Variable("name"),
        Variable("permissions"),
        Variable("roles"),
        accept_expression=True,
    )
    role_definitions = []
    for result in role_resources:
        resource_def = result["bindings"]["resource"]
        assert resource_def.operator == "And"
        assert len(resource_def.args) == 1
        arg = resource_def.args[0]
        type = isa_type(arg)

        name = result["bindings"]["name"]
        permissions = result["bindings"]["permissions"]
        role_defs = result["bindings"]["roles"]

        assert type in oso.host.classes
        python_class = oso.host.classes[type]

        if isinstance(permissions, Variable):
            permissions = []

        # Check for duplicate permissions.
        for perm in permissions:
            if permissions.count(perm) > 1:
                raise OsoError(f"Duplicate action {perm} for resource {type}")

        if isinstance(role_defs, Variable):
            role_names = []
        else:
            role_names = role_defs.keys()

        if len(permissions) == 0 and len(role_names) == 0:
            raise OsoError("Must define actions or roles for resource.")

        if name in config.resources:
            raise OsoError(f"Duplicate resource name {name}")

        resource = Resource(
            python_class=python_class,
            type=python_class.__name__,
            name=name,
            actions=permissions,
            roles=role_names,
        )
        config.resources[resource.name] = resource

        permissions = [
            Permission(
                name=action, type=python_class.__name__, python_class=python_class
            )
            for action in permissions
        ]
        for permission in permissions:
            config.permissions.append(permission)

        # Collect up the role definitions to process after we know all the permissions
        role_definitions.append((python_class, role_defs))

    for python_class, role_defs in role_definitions:
        if isinstance(role_defs, Variable):
            continue  # No roles defined

        for name, role_def in role_defs.items():
            if name in config.roles:
                raise OsoError(f"Duplicate role name {name}")

            for key in role_def.keys():
                if key not in ("perms", "implies"):
                    raise OsoError(f"Invalid key in role definition :'{key}'")

            role_permissions = []
            if "perms" in role_def:
                for permission in role_def["perms"]:
                    perm = parse_permission(permission, python_class, config)
                    role_permissions.append(perm)

            implied_roles = []
            if "implies" in role_def:
                implied_roles = role_def["implies"]

            if len(role_permissions) == 0 and len(implied_roles) == 0:
                raise OsoError("Must define permissions or implied roles for a role.")

            role = Role(
                name=name,
                python_class=python_class,
                type=python_class.__name__,
                permissions=role_permissions,
                implied_roles=implied_roles,
            )
            config.roles[role.name] = role

    # Validate config
    for name, role in config.roles.items():
        for permission in role.permissions:
            if permission.python_class != role.python_class:
                for _, other_role in config.roles.items():
                    if other_role.python_class == permission.python_class:
                        raise OsoError(
                            f"Permission {permission.name} on {permission.type} "
                            + f"can not go on role {name} on {role.type} "
                            + f"because {permission.type} has it's own roles. Use an implication."
                        )

                cls = permission.python_class
                while cls != role.python_class:
                    stepped = False
                    for rel in config.relationships:
                        if cls == rel.child_python_class:
                            cls = rel.parent_python_class
                            stepped = True
                            break
                    if not stepped:
                        raise OsoError(
                            f"Permission {permission.name} on {permission.type} "
                            + f"can not go on role {name} on {role.type} "
                            + "because no relationship exists."
                        )

        for implied in role.implied_roles:
            # Make sure implied role exists
            if implied not in config.roles:
                raise OsoError(f"Role '{implied}' implied by '{name}' does not exist.")
            implied_role = config.roles[implied]
            # Make sure implied role is on a valid class
            cls = implied_role.python_class
            while cls != role.python_class:
                stepped = False
                for rel in config.relationships:
                    if cls == rel.child_python_class:
                        cls = rel.parent_python_class
                        stepped = True
                        break
                if not stepped:
                    raise OsoError(
                        f"Role {name} on {role.type} "
                        + f"can not imply role {implied} on {implied_role.type} "
                        + "because no relationship exists."
                    )
            # Make sure implied roles dont have overlapping permissions.
            # @TODO: Follow implication chair further than just one.
            permissions = role.permissions
            for implied_perm in implied_role.permissions:
                if implied_perm in permissions:
                    raise OsoError(
                        f"Invalid implication. Role {role} has permission {implied_perm.name} "
                        + f"on {implied_perm.type} but implies role {implied} "
                        + f"which also has permission {implied_perm.name} on {implied_perm.type}"
                    )

    if len(config.resources) == 0:
        raise OsoError("Need to define resources to use oso roles.")

    return config


def ensure_configured(func):
    def wrapper(self, *args, **kwargs):
        if self.config is None:
            self._read_policy()
        return func(self, *args, **kwargs)

    return wrapper


class OsoRoles:
    def __init__(self, oso, sqlalchemy_base, user_model, session_maker):
        self.session_maker = session_maker

        for cls in session_maker.class_.__mro__:
            if cls.__name__ == "AuthorizedSessionBase":
                raise OsoError(
                    "Must pass a normal session maker not an authorized session maker."
                )

        _check_valid_model(user_model)
        user_pk_name, user_pk_type = get_pk(user_model)
        user_table_name = user_model.__tablename__

        models = sqlalchemy_base._decl_class_registry

        # @NOTE: This is pretty hacky, also will break if the user defines their own classes with these names, so we should
        # make them more unique
        if models.get("UserRole"):
            UserRole = models["UserRole"]
        else:
            # Tables for the management api to save data.
            class UserRole(sqlalchemy_base):
                __tablename__ = "user_roles"
                id = Column(Integer, primary_key=True)
                user_id = Column(
                    user_pk_type, ForeignKey(f"{user_table_name}.{user_pk_name}")
                )
                resource_type = Column(String, index=True)
                resource_id = Column(
                    String, index=True
                )  # Most things can turn into a string lol.
                role = Column(String, index=True)

        if models.get("Permission"):
            Permission = models["Permission"]
        else:

            class Permission(sqlalchemy_base):
                __tablename__ = "permissions"
                id = Column(Integer, primary_key=True)
                resource_type = Column(String, index=True)
                name = Column(String, index=True)

        if models.get("Role"):
            Role = models["Role"]
        else:

            class Role(sqlalchemy_base):
                __tablename__ = "roles"
                name = Column(String, primary_key=True)
                resource_type = Column(String)

        if models.get("RolePermission"):
            RolePermission = models["RolePermission"]
        else:

            class RolePermission(sqlalchemy_base):
                __tablename__ = "role_permissions"
                id = Column(Integer, primary_key=True)
                role = Column(String)
                permission_id = Column(Integer, index=True)

        if models.get("RoleImplication"):
            RoleImplication = models["RoleImplication"]
        else:

            class RoleImplication(sqlalchemy_base):
                __tablename__ = "role_implications"
                id = Column(Integer, primary_key=True)
                from_role = Column(String, index=True)
                to_role = Column(String)

        self._wrapped_oso = oso
        self.UserRole = UserRole
        self.Permission = Permission
        self.Role = Role
        self.RolePermission = RolePermission
        self.RoleImplication = RoleImplication

        class Roles:
            @staticmethod
            def role_allows(user, action, resource):
                if self.config is None:
                    self._read_policy()
                return self._role_allows(user, action, resource)

            @staticmethod
            def user_in_role(user, role, resource):
                if self.config is None:
                    self._read_policy()
                return self._user_in_role(user, role, resource)

        self.config = None
        self.synced = False
        self._wrapped_oso.register_class(Roles)

    def _get_session(self):
        return self.session_maker()

    def _read_policy(self):
        self.config = read_config(self._wrapped_oso)

    @ensure_configured
    def synchronize_data(self, session=None):
        """
        Call to load the roles data from the policy to the database so that it can be
        evaluated. This must be called every time the policy changes, usually as part
        of a deploy script.
        """
        # Sync static data to the database.
        if session is None:
            session = self._get_session()

        session.execute("delete from role_permissions")
        session.execute("delete from role_implications")
        session.execute("delete from roles")
        session.execute("delete from permissions")

        permissions = {}
        for p in self.config.permissions:
            name = p.name
            type = p.type
            permissions[(name, type)] = self.Permission(resource_type=type, name=name)

        for _, p in permissions.items():
            session.add(p)

        session.flush()

        roles = []
        role_permissions = []
        role_implications = []
        for _, role in self.config.roles.items():
            roles.append(self.Role(name=role.name, resource_type=role.type))
            for permission in role.permissions:
                perm_name = permission.name
                perm_type = permission.type
                perm_key = (perm_name, perm_type)
                assert perm_key in permissions
                perm = permissions[perm_key]
                role_permissions.append(
                    self.RolePermission(role=role.name, permission_id=perm.id)
                )
            for implies in role.implied_roles:
                role_implications.append(
                    self.RoleImplication(from_role=role.name, to_role=implies)
                )

        for role in roles:
            session.add(role)

        for rp in role_permissions:
            session.add(rp)

        for ri in role_implications:
            session.add(ri)

        session.commit()

        id_query = "case resources.type\n"
        type_query = "case resources.type\n"
        child_types = []

        self.role_allow_list_filter_queries = {}
        self.user_in_role_list_filter_queries = {}

        # @NOTE: WOW HACK
        for relationship in self.config.relationships:
            parent_id_field = (
                inspect(relationship.parent_python_class).primary_key[0].name
            )
            child_id_field = (
                inspect(relationship.child_python_class).primary_key[0].name
            )

            parent_id = parent_id_field
            parent_table = relationship.parent_table
            parent_type = relationship.parent_type
            parent_join_column = relationship.parent_join_column
            child_id = child_id_field
            child_table = relationship.child_table
            child_type = relationship.child_type
            child_join_column = relationship.child_join_column
            select = f"""
                select p.{parent_id}
                from {child_table} c
                join {parent_table} p
                on cast(c.{child_join_column} as text) = cast(p.{parent_join_column} as text)
                where c.{child_id} = resources.id"""

            id_query += ""

            id_query += f"when '{child_type}' then (\n"
            id_query += select
            id_query += "\n)\n"

            type_query += f"when '{child_type}' then '{parent_type}'\n"

            child_types.append(child_type)

        id_query += "end as id"
        type_query += "end as type"

        resource_id_field = "cast(:resource_id as text)"

        has_relationships = len(self.config.relationships) > 0
        self.role_allow_sql_query = role_allow_query(
            id_query, type_query, child_types, resource_id_field, has_relationships
        )

        self.user_in_role_sql_query = user_in_role_query(
            id_query, type_query, child_types, resource_id_field, has_relationships
        )

        for _, resource in self.config.resources.items():
            python_class = resource.python_class
            type = resource.type
            id_field = inspect(python_class).primary_key[0].name
            table = python_class.__tablename__
            self.role_allow_list_filter_queries[
                type
            ] = f"""
                select
                  {id_field}
                from {table} R
                where exists (
                  {role_allow_query(id_query, type_query, child_types, "cast(R."+id_field+" as text)", has_relationships)}
                )
            """
            self.user_in_role_list_filter_queries[
                type
            ] = f"""
                select
                  {id_field}
                from {table} R
                where exists (
                  {user_in_role_query(id_query, type_query, child_types, "cast(R."+id_field+" as text)", has_relationships)}
                )
            """

    def _roles_query(self, user, arg2, resource, query, **kwargs):
        # We shouldn't get any data filtering calls to this method
        if not isinstance(arg2, str):
            raise OsoError("Expected a string, got {}", arg2)

        session = self._get_session()

        try:
            user_pk_name, _ = get_pk(user.__class__)
            user_id = getattr(user, user_pk_name)

            resource_pk_name, _ = get_pk(resource.__class__)
        except sqlalchemy.exc.NoInspectionAvailable:
            # User or Resource is not a sqlalchemy object
            return False

        resource_id = str(getattr(resource, resource_pk_name))

        params = {
            "user_id": user_id,
            "resource_id": resource_id,
            "resource_type": resource.__class__.__name__,
        }
        params.update(kwargs)

        results = session.execute(query, params)
        return bool(results.first())

    def _role_allows(self, user, action, resource):
        return self._roles_query(
            user, action, resource, self.role_allow_sql_query, action=action
        )

    def _user_in_role(self, user, role, resource):
        return self._roles_query(
            user, role, resource, self.user_in_role_sql_query, role=role
        )

    def _get_user_role(self, session, user, resource, role_name):
        """Gets user role for resource if exists"""
        if role_name not in self.config.roles:
            raise OsoError(f"Could not find role {role_name}")

        role = self.config.roles[role_name]

        if not resource.__class__ == role.python_class:
            raise OsoError(
                f'No Role "{role_name}" '
                + "for resource {resource} "
                + "(expected resource to be of type {role.type})."
            )

        user_pk_name, _ = get_pk(user.__class__)
        user_id = getattr(user, user_pk_name)
        resource_type = resource.__class__.__name__
        resource_pk_name, _ = get_pk(resource.__class__)
        resource_id = str(getattr(resource, resource_pk_name))

        user_role = (
            session.query(self.UserRole)
            .filter(
                self.UserRole.user_id == user_id,
                self.UserRole.resource_type == resource_type,
                self.UserRole.resource_id == resource_id,
            )
            .first()
        )

        return user_role

    @ensure_configured
    def assign_role(self, user, resource, role_name, session=None, reassign=True):
        if not session:
            my_session = self._get_session()
        else:
            my_session = session

        user_role = self._get_user_role(my_session, user, resource, role_name)

        if user_role is not None:
            if reassign:
                user_role.role = role_name
            else:
                raise OsoError(
                    f"User {user} already has a role for this resource. To reassign, call with `reassign=True`."
                )
        else:
            user_pk_name, _ = get_pk(user.__class__)
            user_id = getattr(user, user_pk_name)
            resource_type = resource.__class__.__name__
            resource_pk_name, _ = get_pk(resource.__class__)
            resource_id = str(getattr(resource, resource_pk_name))

            user_role = self.UserRole(
                user_id=user_id,
                resource_type=resource_type,
                resource_id=resource_id,
                role=role_name,
            )
            my_session.add(user_role)
        my_session.flush()
        if not session:
            my_session.commit()

    @ensure_configured
    def remove_role(self, user, resource, role_name, session=None):
        if not session:
            my_session = self._get_session()
        else:
            my_session = session

        user_role = self._get_user_role(my_session, user, resource, role_name)

        if user_role is None:
            return False

        my_session.delete(user_role)
        my_session.flush()
        if not session:
            my_session.commit()
        return True

    @ensure_configured
    def for_resource(self, resource_class, session=None):
        # List the roles for a resource type
        roles = []
        for name, role in self.config.roles.items():
            if role.python_class == resource_class:
                roles.append(name)
        return roles

    @ensure_configured
    def assignments_for_resource(self, resource, session=None):
        # List the role assignments for a specific resource
        if not session:
            session = self._get_session()

        resource_type = resource.__class__.__name__
        resource_pk_name, _ = get_pk(resource.__class__)
        resource_id = str(getattr(resource, resource_pk_name))

        user_roles = (
            session.query(self.UserRole)
            .filter(
                self.UserRole.resource_type == resource_type,
                self.UserRole.resource_id == resource_id,
            )
            .all()
        )
        return [{"user_id": ur.user_id, "role": ur.role} for ur in user_roles]

    @ensure_configured
    def assignments_for_user(self, user, session=None):
        # List the role assignments for a specific user
        if not session:
            session = self._get_session()

        user_pk_name, _ = get_pk(user.__class__)
        user_id = getattr(user, user_pk_name)

        user_roles = (
            session.query(self.UserRole)
            .filter(
                self.UserRole.user_id == user_id,
            )
            .all()
        )
        return [
            {
                "resource_type": ur.resource_type,
                "resource_id": ur.resource_id,
                "role": ur.role,
            }
            for ur in user_roles
        ]


def _generate_query_filter(oso, role_method, model):
    user = role_method.args[0]
    action_or_role = role_method.args[1]

    session = oso.roles._get_session()

    try:
        user_pk_name, _ = get_pk(user.__class__)
        user_id = getattr(user, user_pk_name)

        resource_type = model.__name__
        resource_pk_name, _ = get_pk(model)
    except sqlalchemy.exc.NoInspectionAvailable:
        # User or Resource is not a sqlalchemy object
        return sql.false()

    params = {
        "user_id": user_id,
        "resource_type": resource_type,
    }

    try:
        if role_method.name == "role_allows":
            list_sql = oso.roles.role_allow_list_filter_queries[resource_type]
            params["action"] = action_or_role

        elif role_method.name == "user_in_role":
            list_sql = oso.roles.user_in_role_list_filter_queries[resource_type]
            params["role"] = action_or_role

        else:
            # Should never reach here
            raise OsoError(
                "Unexpected role method called with partial resource variable: {}",
                role_method.name,
            )
    except KeyError:
        return sql.false()

    results = session.execute(list_sql, params)
    resource_ids = [id[0] for id in results.fetchall()]
    id_in = getattr(model, resource_pk_name).in_(resource_ids)
    return id_in


def _check_valid_model(*args, raise_error=True):
    for model in args:
        valid = True
        try:
            class_mapper(model)
        except UnmappedClassError:
            valid = False

        if raise_error and not valid:
            raise TypeError(f"Expected a model (mapped class); received: {model}")
