from oso import Oso, OsoRoles
from dataclasses import dataclass


@dataclass
class User:
    name: str


@dataclass
class Organization:
    id: str


@dataclass
class Repository:
    id: str
    org: Organization


def setup():
    # Set up oso
    oso = Oso()
    oso.register_class(User)
    oso.register_class(Organization)

    # Set up roles
    roles = OsoRoles(oso)
    roles.enable()

    policy = """
    allow(_, _, _);
    """

    oso.load_str(policy)

    return oso


def app(oso):
    # Demo data
    osohq = Organization(id="osohq")

    leina = User(name="Leina")
    steve = User(name="Steve")

    assert oso.is_allowed(leina, "invite", osohq)


if __name__ == "__main__":
    oso = setup()
    app(oso)
    print("it works!")
