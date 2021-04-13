from oso import Variable
from dataclasses import dataclass
from typing import Any

# Roles Library

# Set up roles (configure)


# Internal types
@dataclass
class Permission:
    id: int
    resource_type: Any
    action: str


@dataclass
class Role:
    id: int
    resource_type: Any
    name: str


# we chose this datamodel so that user-role assignments will always be to Roles, even if the role becomes scoped,
# so that scoping roles doesn't require modifying the user-role assignments
@dataclass
class ScopedRole:
    id: int
    resource: Any
    role_id: int


@dataclass
class RolePermission:
    id: int
    role_id: int
    permission_id: int


@dataclass
class ScopedRolePermission:
    id: int
    scoped_role_id: int
    permission_id: int


@dataclass
class ParentRelationship:
    id: int
    name: str
    child_type: Any
    parent_type: Any
    parent_selector: Any


# If you have from_role, you also get to_role
# add types probably
@dataclass
class ImpliedRole:
    id: int
    from_role_id: int
    to_role_id: int


# scoped the second most specifically, scoped for all children of the parent
class ParentImpliedRole:
    id: int
    parent_id: int
    from_role: int  # nullable
    to_role: int  # nullable


# scoped the most specifically, for a single child record
class ParentChildImpliedRole:
    id: int
    parent_id: int
    child_id: int
    from_role: int  # nullable
    to_role: int  # nullable


@dataclass
class UserRole:
    id: int
    user: Any
    resource: Any
    role_id: int


# Roles api from polar

# TODO: A nice way to add indexes to this.
class Collection:
    def __init__(self):
        self.elements = {}
        self.next_id = 0

    def get_id(self):
        id = self.next_id
        self.next_id += 1
        return id


def ensure_configured(func):
    def wrapper(self, *args, **kwargs):
        if not self.configured:
            self._configure()
        func(self, *args, **kwargs)

    return wrapper


class OsoRoles:
    def __init__(self, oso):
        self.parent_relationships = Collection()
        self.permissions = Collection()
        self.roles = Collection()
        self.scoped_roles = Collection()
        self.role_permissions = Collection()
        self.scoped_role_permissions = Collection()
        self.implied_roles = Collection()
        self.parent_implied_roles = Collection()
        self.parent_child_implied_roles = Collection()
        self.user_roles = Collection()
        self.types = {}
        self.role_names = {}
        # self.permission_names = {}

        self.oso = oso
        self.configured = False

    def register_class(self, type):
        self.types[type.__name__] = type

    def _new_relationship(self, name, child, parent, parent_selector):
        id = self.parent_relationships.get_id()
        relationship = ParentRelationship(
            id=id,
            name=name,
            parent_type=parent,
            child_type=child,
            parent_selector=parent_selector,
        )
        self.parent_relationships.elements[id] = relationship
        return relationship

    def _new_permission(self, resource, action):
        id = self.permissions.get_id()
        permission = Permission(id=id, resource_type=resource, action=action)
        self.permissions.elements[id] = permission
        return permission

    def _new_role(self, resource, name):
        id = self.roles.get_id()
        role = Role(id=id, resource_type=resource, name=name)
        self.roles.elements[id] = role
        return role

    # TODO: scoped roles

    def _are_types_related(self, child_type, parent_type):
        # Check if child type is related to parent_type
        current_resource_type = child_type
        stepped = True
        while stepped:
            stepped = False
            for _, relationship in self.parent_relationships.elements.items():
                if relationship.child_type == current_resource_type:
                    current_resource_type = relationship.parent_type
                    stepped = True
                    break
            if current_resource_type == parent_type:
                return True
        return False

    def _add_role_permission(self, role, permission):
        assert isinstance(role, Role)
        assert isinstance(permission, Permission)

        assert role.id in self.roles.elements
        assert permission.id in self.permissions.elements

        # If resources don't match, ensure there's a relationship.
        if permission.resource_type != role.resource_type:
            assert self._are_types_related(permission.resource_type, role.resource_type)

            # If permission is on a child type, ensure there's no roles for that child type.
            for _, role in self.roles.elements.items():
                if role.resource_type == permission.resource_type:
                    # TODO: Error, can't assign a permission to a parent role if the type the permission is on has
                    # roles.
                    assert False

        id = self.role_permissions.get_id()
        role_permission = RolePermission(
            id=id, role_id=role.id, permission_id=permission.id
        )
        self.role_permissions.elements[id] = role_permission

        return role_permission

    @ensure_configured
    def add_scoped_role_permission(self, scope, role_name, perm_name):
        role = self.role_names[role_name]
        assert isinstance(role, Role)
        assert role.id in self.roles.elements

        assert ":" in perm_name
        permission = self.permission_names[perm_name]
        assert isinstance(permission, Permission)
        assert permission.id in self.permissions.elements

        # If resources don't match, ensure there's a relationship.
        if permission.resource_type != role.resource_type:
            assert self._are_types_related(permission.resource_type, role.resource_type)

            # If permission is on a child type, ensure there's no roles for that child type.
            for _, role in self.roles.elements.items():
                if role.resource_type == permission.resource_type:
                    # TODO: Error, can't assign a permission to a parent role if the type the permission is on has
                    # roles.
                    assert False

        # If there is not yet a scoped role, create one.
        scoped_role = None
        for _, sr in self.scoped_roles.elements.items():
            if sr.resource == scope and sr.role_id == role.id:
                scoped_role = sr
                break

        if not scoped_role:
            sr_id = self.scoped_roles.get_id()
            scoped_role = ScopedRole(id=sr_id, resource=scope, role_id=role.id)
            self.scoped_roles.elements[sr_id] = scoped_role

            # Copy the permissions from the role to the scoped role.
            for _, role_permission in self.role_permissions.elements.items():
                if role_permission.role_id == scoped_role.role_id:
                    id = self.scoped_role_permissions.get_id()
                    scoped_role_permission = ScopedRolePermission(
                        id=id,
                        scoped_role_id=scoped_role.id,
                        permission_id=role_permission.permission_id,
                    )
                    self.scoped_role_permissions.elements[id] = scoped_role_permission

        id = self.scoped_role_permissions.get_id()
        role_permission = ScopedRolePermission(
            id=id, scoped_role_id=scoped_role.id, permission_id=permission.id
        )
        self.scoped_role_permissions.elements[id] = role_permission
        return role_permission

    # TODO: delete scoped role permissions
    @ensure_configured
    def remove_scoped_role_permission(self, scope, role, permission):
        pass

    def _add_role_implies(self, from_role, to_role):
        # @TODO:
        # If resources don't match, ensure there's a relationship.
        # Two mutually exclusive roles can not be implied by the same role.

        assert isinstance(from_role, Role)
        assert isinstance(to_role, Role)

        assert from_role.id in self.roles.elements
        assert to_role.id in self.roles.elements

        # If resources don't match, ensure there's a relationship.
        if from_role.resource_type != to_role.resource_type:
            assert self._are_types_related(
                to_role.resource_type, from_role.resource_type
            )

        id = self.implied_roles.get_id()
        implied_role = ImpliedRole(
            id=id, from_role_id=from_role.id, to_role_id=to_role.id
        )
        self.implied_roles.elements[id] = implied_role

        return implied_role

    # TODO: Scoped implied roles (by parent)
    # TODO: Scoped implied roles (by parent and child)

    # TODO: Remove implied roles

    # Start of the "dynamic api"

    @ensure_configured
    def assign_role(self, user, resource, role_name):
        # @TODO:
        # Can't be assigned to two different mutually exclusive roles.
        # Role has to be on the resource.
        role = self.role_names[role_name]

        assert isinstance(role, Role)
        assert role.id in self.roles.elements

        id = self.user_roles.get_id()
        user_role = UserRole(id=id, user=user, resource=resource, role_id=role.id)
        self.user_roles.elements[id] = user_role

        return user_role

    # TODO: Update role
    # TODO: Remove role

    # Internal api for evaluation of stuff
    def _role_allows(self, user, action, resource):
        # a user is aloud to take an action on a resource if they have
        # permission to.
        # That permission comes from a role.
        # That role comes from a direct assignment to a role with the permission
        # or assignment to a role that implies a role with the permission.

        # Get all the related resources
        resources = {resource.__class__: resource}
        current_resource = resource
        current_resource_type = resource.__class__
        stepped = True
        while stepped:
            stepped = False
            for _, relationship in self.parent_relationships.elements.items():
                if relationship.child_type == current_resource_type:
                    current_resource = relationship.parent_selector(current_resource)
                    current_resource_type = current_resource.__class__
                    assert current_resource_type == relationship.parent_type
                    resources[current_resource_type] = current_resource
                    stepped = True
                    break

        # Find the permission.
        permission = None
        for _, perm in self.permissions.elements.items():
            if perm.resource_type == resource.__class__ and perm.action == action:
                permission = perm
                break
        if not permission:
            return False

        # Go through all scoped role permissions
        # Get any scoped roles with this permission
        scoped_role_ids = set()
        for _, scoped_role_perm in self.scoped_role_permissions.elements.items():
            if scoped_role_perm.permission_id == permission.id:
                scoped_role = self.scoped_roles.elements[
                    scoped_role_perm.scoped_role_id
                ]
                if scoped_role.resource in resources.values():
                    scoped_role_ids.add(scoped_role.role_id)

        # go through all role permissions
        #  get any roles with this permission if there isn't a scoped role for it for these resources
        base_role_ids = set()
        for _, role_perm in self.role_permissions.elements.items():
            if role_perm.permission_id == permission.id:
                current_role_id = role_perm.role_id
                # check if current role is scoped to any relevant resources
                found_scoped_role = False
                for _, scoped_role in self.scoped_roles.elements.items():
                    if (
                        scoped_role.role_id == current_role_id
                        and scoped_role.resource in resources.values()
                    ):
                        # if the role is scoped, the default role doesn't apply
                        found_scoped_role = True
                        break
                if not found_scoped_role:
                    base_role_ids.add(current_role_id)

        assert scoped_role_ids.isdisjoint(base_role_ids)
        role_ids = scoped_role_ids.union(base_role_ids)

        # follow implications following scoped implied rules
        while True:
            size = len(role_ids)

            for _, implied_role in self.implied_roles.elements.items():

                new_role_ids = set()
                for role_id in role_ids:
                    if implied_role.to_role_id == role_id:
                        new_role_ids.add(implied_role.from_role_id)

                role_ids = role_ids.union(new_role_ids)

            if len(role_ids) == size:
                break

        # See if the user is assigned to any of those roles
        for _, user_role in self.user_roles.elements.items():
            for role_id in role_ids:
                if (
                    user_role.role_id == role_id
                    and user_role.user == user
                    and user_role.resource in resources.values()
                ):
                    return True

        return False

    def _configure(self):
        # Note(steve)
        # This is all just hacked to get the policy to work.

        # Register relationships
        role_relationships = self.oso.query_rule(
            "parent",
            Variable("resource"),
            Variable("parent_resource"),
            accept_expression=True,
        )
        relationships = []
        for result in role_relationships:
            # OMG WOW HACK, OMFG WOW HACK
            # will not work in general lol but looks like
            # it works for the demo.
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
            self._new_relationship(
                name=f"{child_t}_{parent_t}",
                child=self.types[child_t],
                parent=self.types[parent_t],
                parent_selector=lambda child: getattr(child, parent_field),
            )

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
        resources = []
        for result in role_resources:
            resource = result["bindings"]["resource"]
            assert resource.operator == "And"
            assert len(resource.args) == 1
            arg = resource.args[0]
            assert arg.operator == "Isa"
            assert len(arg.args) == 2
            assert arg.args[0] == Variable("_this")
            pattern = arg.args[1]
            t = pattern.tag
            name = result["bindings"]["name"]
            permissions = result["bindings"]["permissions"]
            roles = result["bindings"]["roles"]

            resources.append(
                {"type": t, "permissions": permissions, "roles": roles, "name": name}
            )

        permissions = {}
        # Register permissions
        for resource in resources:
            type = resource["type"]
            name = resource["name"]
            for perm in resource["permissions"]:
                permissions[f"{name}:{perm}"] = self._new_permission(
                    resource=self.types[type], action=perm
                )
        self.permission_names = permissions

        # Register roles
        roles = {}
        for resource in resources:
            type = resource["type"]
            resource_name = resource["name"]
            role_list = resource["roles"]
            if isinstance(role_list, dict):
                for role_name, role_data in role_list.items():
                    # WOW HACK, not ideal...
                    name = role_name.split("_", 1)[1]
                    role = self._new_role(resource=self.types[type], name=name)
                    roles[role_name] = role
                    for perm in role_data["perms"]:
                        # either colon namespaces or on this resource
                        if not ":" in perm:
                            perm = f"{resource_name}:{perm}"
                        self._add_role_permission(
                            role=role, permission=permissions[perm]
                        )
        self.role_names = roles

        implications = {}
        # Register implications
        for resource in resources:
            type = resource["type"]
            role_list = resource["roles"]
            if isinstance(role_list, dict):
                for role_name, role_data in role_list.items():
                    if "implies" in role_data:
                        for implies in role_data["implies"]:
                            self._add_role_implies(roles[role_name], roles[implies])

        self.configured = True

    def enable(self):
        # The "Polar api"
        class Roles:
            @staticmethod
            def role_allows(user, action, resource):
                if not self.configured:
                    self._configure(self)
                return self._role_allows(user, action, resource)

        self.oso.register_class(Roles)
