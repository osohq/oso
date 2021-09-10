from dataclasses import dataclass
from typing import Set, Union

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
[oso.register_class(cls) for cls in [Organization, Repository, User]]
oso.load_files(["main.polar"])

ariana = User("ariana", set())
bhavik = User("bhavik", set())

alpha_association = Organization("Alpha Association")
beta_business = Organization("Beta Business")

ariana.assign_role_for_resource("owner", alpha_association)

affine_types = Repository("Affine Types", alpha_association)
allocator = Repository("Allocator", alpha_association)

bubble_sort = Repository("Bubble Sort", beta_business)
benchmarks = Repository("Benchmarks", beta_business)

bhavik.assign_role_for_resource("contributor", bubble_sort)
bhavik.assign_role_for_resource("maintainer", benchmarks)

assert oso.is_allowed(ariana, "read", affine_types)
assert oso.is_allowed(ariana, "push", affine_types)
assert oso.is_allowed(ariana, "read", allocator)
assert oso.is_allowed(ariana, "push", allocator)
assert not oso.is_allowed(ariana, "read", bubble_sort)
assert not oso.is_allowed(ariana, "push", bubble_sort)
assert not oso.is_allowed(ariana, "read", benchmarks)
assert not oso.is_allowed(ariana, "push", benchmarks)

assert not oso.is_allowed(bhavik, "read", affine_types)
assert not oso.is_allowed(bhavik, "push", affine_types)
assert not oso.is_allowed(bhavik, "read", allocator)
assert not oso.is_allowed(bhavik, "push", allocator)
assert oso.is_allowed(bhavik, "read", bubble_sort)
assert not oso.is_allowed(bhavik, "push", bubble_sort)
assert oso.is_allowed(bhavik, "read", benchmarks)
assert oso.is_allowed(bhavik, "push", benchmarks)
