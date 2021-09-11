from dataclasses import dataclass
from typing import Set, Union
from pathlib import Path

from oso import Oso


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


oso = Oso()
oso.register_class(Organization)
oso.register_class(Repository)
oso.register_class(User)
oso.load_files([Path(__file__).parent / "main.polar"])
