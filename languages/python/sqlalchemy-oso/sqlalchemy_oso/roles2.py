# Roles 2
from typing import Any, List, Set
from dataclasses import dataclass

from oso import Variable

from sqlalchemy.types import Integer, String
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy import inspect


# Starting with something like this, tbd


@dataclass
class Relationship:
    child_python_class: Any
    child_type: str
    child_table: str
    parent_python_class: Any
    parent_type: str
    parent_table: str
    parent_selector: Any
    parent_field: str


@dataclass
class Permission:
    python_class: Any
    name: str


@dataclass
class Role:
    name: str
    python_class: Any
    permissions: List[Permission]
    implied_roles: List[str]


@dataclass
class Resource:
    name: str
    python_class: Any
    actions: Set[str]
    roles: Set[str]


def ensure_configured(func):
    def wrapper(self, *args, **kwargs):
        if not self.configured:
            self._configure()
        func(self, *args, **kwargs)

    return wrapper


class OsoRoles:
    def __init__(self, oso, sqlalchemy_base, user_model):
        # @Q: Is this where I should create the models?
        # Would this even work? Do these have to happen at a
        # certain time or something to get created in the database?

        self.session = None

        user_pk_type = inspect(user_model).primary_key[0].type
        user_pk_name = inspect(user_model).primary_key[0].name
        user_table_name = user_model.__tablename__

        # Tables for the management api to save data.
        class UserRole(sqlalchemy_base):
            __tablename__ = "user_roles"
            id = Column(Integer, primary_key=True)
            user_id = Column(
                user_pk_type, ForeignKey(f"{user_table_name}.{user_pk_name}")
            )
            resource_type = Column(String)
            resource_id = Column(String)  # Most things can turn into a string lol.
            role = Column(String)

        # Tables to sync static data to so we can join to it.
        class Relationship(sqlalchemy_base):
            __tablename__ = "relationships"
            id = Column(Integer, primary_key=True)
            child_type = Column(String)
            child_table = Column(String)
            parent_type = Column(String)
            parent_table = Column(String)
            parent_field = Column(String)

        class Permission(sqlalchemy_base):
            __tablename__ = "permissions"
            id = Column(Integer, primary_key=True)
            resource_type = Column(String)
            name = Column(String)

        class Role(sqlalchemy_base):
            __tablename__ = "roles"
            name = Column(String, primary_key=True)
            resource_type = Column(String)

        class RolePermission(sqlalchemy_base):
            __tablename__ = "role_permissions"
            id = Column(Integer, primary_key=True)
            role = Column(String, ForeignKey("roles.name"))
            permission_id = Column(Integer, ForeignKey("permissions.id"))

        class RoleImplication(sqlalchemy_base):
            __tablename__ = "role_implications"
            id = Column(Integer, primary_key=True)
            from_role = Column(String, ForeignKey("roles.name"))
            to_role = Column(String, ForeignKey("roles.name"))


        self.oso = oso
        self.Relationship = Relationship
        self.UserRole = UserRole
        self.Permission = Permission
        self.Role = Role
        self.RolePermission = RolePermission
        self.RoleImplication = RoleImplication

        self.resources = {}
        self.permissions = []
        self.roles = {}
        self.relationships = []

        self.configured = False

    def _configure(self):
        # @TODO: ALLLLL of the validation needed for the role model.

        # @TODO: Figure out where this session should really come from.
        assert self.session

        self.resources = {}
        self.permissions = []
        self.roles = {}
        self.relationships = []

        # Register relationships
        role_relationships = self.oso.query_rule(
            "parent",
            Variable("resource"),
            Variable("parent_resource"),
            accept_expression=True,
        )

        for result in role_relationships:
            # OMG WOW HACK, OMFG WOW HACK
            # will not work in general lol
            constraints = result["bindings"]["resource"]
            assert len(constraints.args) == 2
            type_check = constraints.args[0]
            assert type_check.operator == "Isa"
            assert len(type_check.args) == 2
            assert type_check.args[0] == Variable("_this")
            pattern = type_check.args[1]
            child_t = pattern.tag
            get_parent = constraints.args[1]
            assert get_parent.operator == "Isa"
            assert len(get_parent.args) == 2
            getter = get_parent.args[0]
            assert getter.operator == "Dot"
            assert len(getter.args) == 2
            assert getter.args[0] == Variable("_this")
            parent_field = getter.args[1]
            pattern = get_parent.args[1]
            parent_t = pattern.tag

            # @NOTE: Works around a problem with lambda referencing local
            # vars.
            def make_getter(field):
                def get_parent(child):
                    return getattr(child, field)

                return get_parent

            child_python_class = self.oso.host.classes[child_t]
            child_table = child_python_class.__tablename__
            parent_python_class = self.oso.host.classes[parent_t]
            parent_table = parent_python_class.__tablename__

            relationship = Relationship(
                child_python_class=child_python_class,
                child_type=child_t,
                child_table=child_table,
                parent_python_class=parent_python_class,
                parent_type=parent_t,
                parent_table=parent_table,
                parent_selector=make_getter(parent_field),
                parent_field=parent_field
            )

            self.relationships.append(relationship)

        # Register resources / permissions / roles and implications
        # Based on the role_resource definitions
        role_resources = self.oso.query_rule(
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
            assert arg.operator == "Isa"
            assert len(arg.args) == 2
            assert arg.args[0] == Variable("_this")
            pattern = arg.args[1]
            t = pattern.tag
            name = result["bindings"]["name"]
            permissions = result["bindings"]["permissions"]
            role_defs = result["bindings"]["roles"]

            python_class = (self.oso.host.classes[t],)

            if isinstance(permissions, Variable):
                permissions = set()
            else:
                permissions = set(permissions)

            if isinstance(role_defs, Variable):
                role_names = set()
            else:
                role_names = set(role_defs.keys())

            resource = Resource(
                python_class=python_class,
                name=name,
                actions=permissions,
                roles=role_names,
            )
            self.resources[resource.name] = resource

            permissions = [
                Permission(name=action, python_class=python_class)
                for action in permissions
            ]
            for permission in permissions:
                self.permissions.append(permission)

            role_definitions.append((python_class, role_defs))

        for python_class, role_defs in role_definitions:
            if not isinstance(role_defs, Variable):
                for name, role_def in role_defs.items():
                    role_permissions = []
                    if "perms" in role_def:
                        for permission in role_def["perms"]:
                            if ":" in permission:
                                resource_name, action = permission.split(":", 1)
                                assert resource_name in self.resources
                                permission_python_class = self.resources[
                                    resource_name
                                ].python_class
                            else:
                                action = permission
                                permission_python_class = python_class
                            role_permissions.append(
                                Permission(
                                    name=action, python_class=permission_python_class
                                )
                            )

                    implied_roles = []
                    if "implies" in role_def:
                        implied_roles = role_def["implies"]

                    role = Role(
                        name=name,
                        python_class=python_class,
                        permissions=role_permissions,
                        implied_roles=implied_roles,
                    )
                    self.roles[role.name] = role

        # Sync static data to the database.
        self.session.execute("delete from relationships")
        self.session.execute("delete from role_permissions")
        self.session.execute("delete from role_implications")
        self.session.execute("delete from roles")
        self.session.execute("delete from permissions")

        relationships = [self.Relationship(child_type=r.child_type, child_table=r.child_table, parent_type=r.parent_type, parent_table=r.parent_table, parent_field=r.parent_field) for r in self.relationships]
        for r in relationships:
            self.session.add(r)

        permissions = {}
        for p in self.permissions:
            name = p.name
            t = str(p.python_class[0].__name__)
            permissions[(name, t)] = self.Permission(resource_type=t, name=name)

        for _, p in permissions.items():
            self.session.add(p)

        self.session.commit()

        roles = []
        role_permissions = []
        role_implications = []
        for _, role in self.roles.items():
            roles.append(self.Role(name=role.name, resource_type=role.python_class[0].__name__))
            for permission in role.permissions:
                perm_name = permission.name
                perm_type = str(permission.python_class[0].__name__)
                perm_key = (perm_name, perm_type)
                assert perm_key in permissions
                perm = permissions[perm_key]
                role_permissions.append(self.RolePermission(role=role.name, permission_id=perm.id))
            for implies in role.implied_roles:
                role_implications.append(self.RoleImplication(from_role=role.name, to_role=implies))

        for role in roles:
            self.session.add(role)

        for rp in role_permissions:
            self.session.add(rp)

        for ri in role_implications:
            self.session.add(ri)

        self.session.commit()

        self.configured = True

    def _role_allows(self, user, action, resource):
        # @TODO: Figure out where this session should really come from.
        assert self.session

        # @NOTE: First pass at this does things in a slow way that won't really scale to having to do
        # it all in a single sql query which is necessary for the list processing cases.
        # This first pass is important as a baseline for the evaluation.

        # A user is aloud to take an action on a resource if they have permission to.
        # That permission comes from a role.
        # That role comes from one of
        # - a direct assignment of the user to the role with the permission, or
        # - assignment of the user to a role that implies the role with the permission

        # The permission can be on a role for the passed in resource or a role
        # for any resource that's an ancestor of the passed in resource.
        # We can use the defined relationships to "walk up" the tree of resources
        # to get all the ancestor resources.
        # We'll need to know those so we can check if the user is assigned to roles
        # on them so we'll start by fetching them all.
        relevent_resources = {resource.__class__: resource}
        current_resource = resource
        current_resource_class = resource.__class__
        stepped = True
        while stepped:
            stepped = False
            for relationship in self.relationships:
                if relationship.child_python_class == current_resource_class:
                    current_resource = relationship.parent_selector(current_resource)
                    current_resource_class = current_resource.__class__
                    assert current_resource_class == relationship.parent_python_class
                    relevent_resources[current_resource_class] = current_resource
                    stepped = True
                    break

        # Now we can try to resolve the allow call.
        #
        # Start by finding the permission for this action on this resource type.
        permission = None
        for perm in self.permissions:
            if resource.__class__ in perm.python_class and action == perm.name:
                permission = perm
                break
        if not permission:
            return False

        # @TODO: Check scoped role permissions for any scoped roles with this
        # permission.

        # Go through all static roles to see if they contain this permission.
        relevant_roles = {}
        for role_name, role in self.roles.items():
            for role_perm in role.permissions:
                if (
                    role_perm.python_class == permission.python_class
                    and role_perm.name == permission.name
                ):
                    # TODO: Check if the role is scoped to any relevant resources.
                    relevant_roles[role_name] = role

        # Recursively add any roles that imply a relevant role to relevant_roles
        # @TODO: Check scoped implications.
        while True:
            size = len(relevant_roles)
            for role_name, role in self.roles.items():
                for implied_role in role.implied_roles:
                    if implied_role in relevant_roles:
                        relevant_roles[role_name] = role
                        break
            if len(relevant_roles) == size:
                break

        # Now we have all the roles the user could be assigned to that would give them
        # this permission.
        # Check if the user is in any of these roles.

        # First pass just get all the roles the user is assigned to and then compare.
        user_pk_name = inspect(user.__class__).primary_key[0].name
        user_id = getattr(user, user_pk_name)

        user_roles = self.session.query(self.UserRole).filter_by(user_id=user_id).all()

        for _, relevent_resource in relevent_resources.items():
            resource_type = relevent_resource.__class__.__name__
            resource_pk_name = inspect(relevent_resource.__class__).primary_key[0].name
            resource_id = str(getattr(relevent_resource, resource_pk_name))

            for user_role in user_roles:
                for role_name, role in relevant_roles.items():
                    if (
                        user_role.resource_type == resource_type
                        and user_role.resource_id == resource_id
                        and user_role.role == role_name
                    ):
                        return True

        return False

    def enable(self):
        class Roles:
            @staticmethod
            def role_allows(user, action, resource):
                if not self.configured:
                    self._configure()
                return self._role_allows(user, action, resource)

        self.oso.register_class(Roles)

    @ensure_configured
    def assign_role(self, session, user, resource, role_name):
        # @TODO: Verify all the rules of what roles you can be assigned to.
        assert role_name in self.roles
        role = self.roles[role_name]

        assert resource.__class__ in role.python_class

        user_pk_name = inspect(user.__class__).primary_key[0].name
        user_id = getattr(user, user_pk_name)
        resource_type = resource.__class__.__name__
        resource_pk_name = inspect(resource.__class__).primary_key[0].name
        resource_id = str(getattr(resource, resource_pk_name))

        user_role = self.UserRole(
            user_id=user_id,
            resource_type=resource_type,
            resource_id=resource_id,
            role=role_name,
        )
        session.add(user_role)
