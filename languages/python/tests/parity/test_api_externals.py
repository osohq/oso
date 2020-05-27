# External class definitions for use in `test_api.py` tests
from dataclasses import dataclass

# Fake global actor name → company ID map.
# Should be an external database lookup.
actors = {
    "guest": "1",
    "president": "1",
}

frobbed = []


def get_frobbed():
    global frobbed
    return frobbed


def set_frobbed(f):
    global frobbed
    frobbed = f


# example of registering a non-dataclass class
class Widget:
    # Data fields.
    id: str = ""
    name: str = ""

    # Class variables.
    actions = ("get", "create")

    def __init__(self, id="", name=""):
        self.id = id
        self.name = name

    def company(self):
        yield Company(id=self.id)

    def frob(self, what):
        global frobbed
        frobbed.append(what)
        yield self

    def from_polar(id, name=""):
        return Widget(id, name)


class DooDad(Widget):
    def from_polar(id, name=""):
        return DooDad(id, name)


@dataclass  # NOTE: These are now optional, but here for compat
class Actor:
    name: str = ""
    id: int = 0
    widget: Widget = None

    def company(self):
        yield Company(id="0")  # fake, will fail
        yield Company(id=actors[self.name])  # real, will pass

    def group(self):
        return ["social", "dev", "product"]

    def companies_iter(self):
        return iter([Company(id="acme"), Company(id="Initech")])


@dataclass(frozen=True)
class Company:
    # Data fields.
    id: str = ""
    default_role: str = ""

    # Class variables.
    roles = ("guest", "admin")

    def role(self, actor: Actor):
        if actor.name == "president":
            yield "admin"
        else:
            yield "guest"

    def from_polar(id, default_role):
        return Company(id, default_role)
