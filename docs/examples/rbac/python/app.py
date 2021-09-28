from dataclasses import dataclass
from typing import Set, Union

from oso import Oso


# docs: begin-types
@dataclass(frozen=True)
class Organization:
    name: str


@dataclass(frozen=True)
class Repository:
    name: str
    organization: Organization


@dataclass(frozen=True)
class Role:
    name: str
    resource: Union[Repository, Organization]


@dataclass
class User:
    name: str
    roles: Set[Role]

    def assign_role_for_resource(self, name, resource):
        self.roles.add(Role(name, resource))
        # docs: end-types


# docs: begin-setup
oso = Oso()

# docs: begin-register
oso.register_class(Organization)
oso.register_class(Repository)
oso.register_class(User)
# docs: end-register

oso.load_files(["main.polar"])
# docs: end-setup
