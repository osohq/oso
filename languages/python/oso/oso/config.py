from typing import Any, List, Set, Dict
from dataclasses import dataclass
from polar import Variable
from polar.exceptions import OsoError


@dataclass
class Relationship:
    child_type: str
    parent_type: str


@dataclass
class Permission:
    type: str
    name: str


@dataclass
class Role:
    name: str
    type: str
    permissions: List[Permission]
    implied_roles: List[str]


@dataclass
class Resource:
    type: str
    name: str
    actions: Set[str]
    roles: Set[str]


@dataclass
class Config:
    resources: Dict[str, Resource]
    type_to_resource_name: Dict[str, str]
    permissions: List[Permission]
    roles: Dict[str, Role]
    relationships: List[Relationship]


def isa_type(arg):
    assert arg.operator == "Isa"
    assert len(arg.args) == 2
    assert arg.args[0] == Variable("_this")
    pattern = arg.args[1]
    type = pattern.tag
    return type


def parse_permission(permission, type, config):
    """Parse a permission string, check if it's valid and return a Permission"""
    if ":" in permission:
        resource_name, action = permission.split(":", 1)
        if resource_name not in config.resources:
            raise OsoError("Invalid permission namespace.")
        permission_type = config.resources[resource_name].type
    else:
        action = permission
        permission_type = type
    perm = Permission(
        name=action,
        type=permission_type,
    )
    if perm not in config.permissions:
        raise OsoError(
            f"Permission {perm.name} doesn't exist for resource {perm.type}."
        )
    return perm


def parse_role_name(role_name, type, config, other_ok=False):
    """Parse a role name and return a normalized role name (with namspace).

    :param role_name: un-normalized role name
    :type role_name: str
    :param type: type of resource inside which this role was defined
    :type resource_class: Any
    :param config: role config
    :type config: Config
    :param other_ok: Flag to indicate if a namespace other than `resource_name` is allowed, defaults to False
    :type other_ok: bool, optional
    :return: Normalized role name (with namespace)
    :rtype: str
    """
    if type not in config.type_to_resource_name:
        raise OsoError(f"Unrecognized resource type {type}.")
    resource_name = config.type_to_resource_name[type]
    if ":" in role_name:
        namespace, _ = role_name.split(":", 1)
        if namespace not in config.resources:
            raise OsoError(f"Invalid role namespace {namespace}.")
    else:
        role_name = f"{resource_name}:{role_name}"

    return role_name


def validate_config(oso):
    config = Config(
        resources={},
        type_to_resource_name={},
        permissions=[],
        roles={},
        relationships=[],
    )

    # role_relationships = oso.query_rule(
    #     "parent",
    #     Variable("resource"),
    #     Variable("parent_resource"),
    #     accept_expression=True,
    # )
    # for result in role_relationships:
    #     try:
    #         _constraints = result["bindings"]["resource"]
    #         parent_constraints = result["bindings"]["parent_resource"]
    #         assert len(constraints.args) == 2
    #         type_check = constraints.args[0]
    #         child_type = isa_type(type_check)
    #         get_parent = constraints.args[1]
    #         assert get_parent.operator == "Isa"
    #         assert len(get_parent.args) == 2
    #         getter = get_parent.args[0]
    #         assert getter.operator == "Dot"
    #         assert len(getter.args) == 2
    #         assert getter.args[0] == Variable("_this")
    #         pattern = get_parent.args[1]
    #         parent_type = pattern.tag

    #         relationship = Relationship(child_type=child_type, parent_type=parent_type)
    #         config.relationships.append(relationship)
    #     except AssertionError:
    #         raise OsoError("Invalid relationship.")

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

        resource_name = result["bindings"]["name"]
        permissions = result["bindings"]["permissions"]
        role_defs = result["bindings"]["roles"]

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

        if resource_name in config.resources:
            raise OsoError(f"Duplicate resource name {resource_name}")

        resource = Resource(
            type=type,
            name=resource_name,
            actions=permissions,
            roles=role_names,
        )
        config.resources[resource.name] = resource
        config.type_to_resource_name[type] = resource_name

        permissions = [Permission(name=action, type=type) for action in permissions]
        for permission in permissions:
            config.permissions.append(permission)

        # Collect up the role definitions to process after we know all the permissions
        role_definitions.append((type, role_defs))

    for type, role_defs in role_definitions:
        if isinstance(role_defs, Variable):
            continue  # No roles defined

        for role_name, role_def in role_defs.items():
            # preprocess role name
            role_name = parse_role_name(role_name, type, config)
            if role_name in config.roles:
                raise OsoError(f"Duplicate role name {role_name}")

            for key in role_def.keys():
                if key not in ("permissions", "implies"):
                    raise OsoError(f"Invalid key in role definition :'{key}'")

            role_permissions = []
            if "permissions" in role_def:
                for permission in role_def["permissions"]:
                    perm = parse_permission(permission, type, config)
                    role_permissions.append(perm)

            implied_roles = []
            if "implies" in role_def:
                implied_roles = [
                    parse_role_name(role, type, config, other_ok=True)
                    for role in role_def["implies"]
                ]

            if len(role_permissions) == 0 and len(implied_roles) == 0:
                raise OsoError("Must define permissions or implied roles for a role.")

            role = Role(
                name=role_name,
                type=type,
                permissions=role_permissions,
                implied_roles=implied_roles,
            )
            config.roles[role.name] = role

    # Validate config
    for role_name, role in config.roles.items():
        for permission in role.permissions:
            if permission.type != role.type:
                for _, other_role in config.roles.items():
                    if other_role.type == permission.type:
                        raise OsoError(
                            f"""Permission {permission.name} on {permission.type}
                            can not go on role {role_name} on {role.type}
                            because {permission.type} has it's own roles. Use an implication."""
                        )

                # typ = permission.type
                # while typ != role.type:
                #     stepped = False
                #     for rel in config.relationships:
                #         if typ == rel.child_typ:
                #             typ = rel.parent_typ
                #             stepped = True
                #             break
                #     if not stepped:
                #         raise OsoError(
                #             f"""Permission {permission.name} on {permission.type}
                #             can not go on role {role_name} on {role.type}
                #             because no relationship exists."""
                #         )

        for implied in role.implied_roles:
            # Make sure implied role exists
            if implied not in config.roles:
                raise OsoError(
                    f"Role '{implied}' implied by '{role_name}' does not exist."
                )
            implied_role = config.roles[implied]
            # # Make sure implied role is on a valid class
            # typ = implied_role.type
            # while typ != role.type:
            #     stepped = False
            #     for rel in config.relationships:
            #         if typ == rel.child_type:
            #             typ = rel.parent_type
            #             stepped = True
            #             break
            #     if not stepped:
            #         raise OsoError(
            #             f"""Role {role_name} on {role.type}
            #             can not imply role {implied} on {implied_role.type}
            #             because no relationship exists."""
            #         )
            # Make sure implied roles dont have overlapping permissions.
            # @TODO: Follow implication chair further than just one.
            permissions = role.permissions
            for implied_perm in implied_role.permissions:
                if implied_perm in permissions:
                    raise OsoError(
                        f"""Invalid implication. Role {role} has permission {implied_perm.name}
                        on {implied_perm.type} but implies role {implied}
                        which also has permission {implied_perm.name} on {implied_perm.type}"""
                    )

    if len(config.resources) == 0:
        raise OsoError("Need to define resources to use oso roles.")
