from dataclasses import dataclass
from typing import Set, Union

from oso import Oso


@dataclass(frozen=True)
class Org:
    name: str


@dataclass(frozen=True)
class Repo:
    name: str
    org: Org


@dataclass(frozen=True)
class Role:
    name: str
    resource: Union[Repo, Org]


@dataclass
class User:
    name: str
    roles: Set[Role]

    def assign_role_for_resource(self, name: str, resource: Union[Repo, Org]):
        self.roles.add(Role(name, resource))


oso = Oso()
[oso.register_class(cls) for cls in [Org, Repo, User]]
oso.load_file("./policy.polar")

ariana = User("ariana", set())
bhavik = User("bhavik", set())

alpha_association = Org("Alpha Association")
beta_business = Org("Beta Business")

ariana.assign_role_for_resource("owner", alpha_association)

affine_types_repo = Repo("Affine Types", alpha_association)
allocator_repo = Repo("Allocator", alpha_association)

bubble_sort_repo = Repo("Bubble Sort", beta_business)
breakpoint_repo = Repo("Breakpoint", beta_business)

bhavik.assign_role_for_resource("reader", bubble_sort_repo)
bhavik.assign_role_for_resource("writer", breakpoint_repo)

assert oso.is_allowed(ariana, "pull", affine_types_repo)
assert oso.is_allowed(ariana, "push", affine_types_repo)
assert oso.is_allowed(ariana, "pull", allocator_repo)
assert oso.is_allowed(ariana, "push", allocator_repo)
assert not oso.is_allowed(ariana, "pull", bubble_sort_repo)
assert not oso.is_allowed(ariana, "push", bubble_sort_repo)
assert not oso.is_allowed(ariana, "pull", breakpoint_repo)
assert not oso.is_allowed(ariana, "push", breakpoint_repo)

assert not oso.is_allowed(bhavik, "pull", affine_types_repo)
assert not oso.is_allowed(bhavik, "push", affine_types_repo)
assert not oso.is_allowed(bhavik, "pull", allocator_repo)
assert not oso.is_allowed(bhavik, "push", allocator_repo)
assert oso.is_allowed(bhavik, "pull", bubble_sort_repo)
assert not oso.is_allowed(bhavik, "push", bubble_sort_repo)
assert oso.is_allowed(bhavik, "pull", breakpoint_repo)
assert oso.is_allowed(bhavik, "push", breakpoint_repo)
