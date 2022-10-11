# External class definitions for use in `test_api.py` tests
import re
from dataclasses import dataclass
from typing import List, Optional


class Http:
    """A resource accessed via HTTP."""

    def __init__(self, hostname="", path="", query=None):
        if query is None:
            query = {}
        self.hostname = hostname
        self.path = path
        self.query = query

    def __repr__(self):
        return str(self)

    def __str__(self):
        q = dict(self.query.items())
        host_str = f'hostname="{self.hostname}"' if self.hostname else None
        path_str = f'path="{self.path}"' if self.path != "" else None
        query_str = f"query={q}" if q != {} else None
        field_str = ", ".join(x for x in [host_str, path_str, query_str] if x)
        return f"Http({field_str})"


class PathMapper:
    """Map from a template string with capture groups of the form
    ``{name}`` to a dictionary of the form ``{name: captured_value}``

    :param template: the template string to match against
    """

    def __init__(self, template):
        capture_group = re.compile(r"({([^}]+)})")
        for outer, inner in capture_group.findall(template):
            if inner == "*":
                template = template.replace(outer, ".*")
            else:
                template = template.replace(outer, f"(?P<{inner}>[^/]+)")
        self.pattern = re.compile(f"^{template}$")

    def map(self, string):
        match = self.pattern.match(string)
        if match:
            return match.groupdict()


# Fake global actor name → company ID map.
# Should be an external database lookup.
actors = {"guest": "1", "president": "1"}

frobbed: List[str] = []


def get_frobbed():
    global frobbed
    return frobbed


def set_frobbed(f):
    global frobbed
    frobbed = f


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
        return Company(id=self.id)

    def frob(self, what):
        global frobbed
        frobbed.append(what)
        return self


class DooDad(Widget):
    pass


@dataclass
class User:
    name: str = ""
    id: int = 0
    widget: Optional[Widget] = None

    def companies(self):
        yield Company(id="0")  # fake, will fail
        yield Company(id=actors[self.name])  # real, will pass

    def groups(self):
        return ["social", "dev", "product"]

    def companies_iter(self):
        return iter([Company(id="acme"), Company(id="Initech")])


@dataclass(frozen=True)
class Company:
    # Data fields.
    id: str = ""
    default_role: str = ""

    def role(self, actor: User):
        if actor.name == "president":
            return "admin"
        else:
            return "guest"

    def roles(self):
        yield "guest"
        yield "admin"
