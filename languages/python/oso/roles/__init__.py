from dataclasses import dataclass

# Roles Library

# Set up roles (configure)


# Internal types
@dataclass
class Permission:
    id: int
    resource_type: str
    action: str

@dataclass
class Role:
    id: int
    resource_type: str
    name: str

@dataclass
class ScopedRole:
    id: int
    resource_id: int
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
    parent_class: str
    child_class: str
    parent_field_or_whatever: str # something

# If you have from_role, you also get to_role
# add types probably
@dataclass
class ImpliedRole:
    id: int
    from_role: int
    to_role: int

# scoped the second most specifically, scoped for all children of the parent
class ParentImpliedRole:
    id: int
    parent_id: int
    from_role: int #nullable
    to_role: int   #nullable

# scoped the most specifically, for a single child record
class ParentChildImpliedRole:
    id: int
    parent_id: int
    child_id: int
    from_role: int #nullable
    to_role: int   #nullable

@dataclass
class UserRole:
    id: int
    user_id: str
    role_id: int
    resource_id: int

# Roles api from polar

class OsoRoles:
    def __init__(self):
        self.roles = {
            0: Role(id=0, resource_type="Repository", name="READ"),
            1: Role(id=1, resource_type="Repository", name="WRITE"),
            2: Role(id=2, resource_type="Repository", name="ADMIN")
        }

        self.permissions = {
            0: Permission(id=0, resource_type="Repository", action="read"),
            1: Permission(id=1, resource_type="Repository", action="write"),
            2: Permission(id=2, resource_type="Repository", action="list_issues"),
            3: Permission(id=3, resource_type="Issue", action="read"),
            4: Permission(id=4, resource_type="Issue", action="write"),
        }

        self.parent_relationships = {
            0: ParentRelationship(id=0, parent_class="Repository", child_class="Issue", parent_field_or_whatever="repo")
        }

        self.role_permissions = {
            0: RolePermission(id=0, role_id=0, permission_id=0),
            1: RolePermission(id=1, role_id=0, permission_id=2),
            2: RolePermission(id=2, role_id=0, permission_id=3),
            3: RolePermission(id=3, role_id=1, permission_id=1),
        }

        self.implied_roles = {
            0: ImpliedRole(id=0, from_role=1, to_role=0),
        }


        # test data!
        pass