from dataclasses import dataclass

from oso import polar_class

DEPARTMENT_MEMBERS = {1: {"engineering": ["dhatch"], "executive": ["sam"]}}

# Assign roles by department
ROLES = {
    1: {
        username: dept
        for (dept, dept_users) in DEPARTMENT_MEMBERS[1].items()
        for username in dept_users
    }
}


@polar_class
@dataclass(frozen=True)
class Company:
    id: int = None

    # ...

    def role(self, username):
        yield ROLES[self.id][username]


@polar_class
@dataclass
class Actor:
    username: str = ""


def load(kb):
    """Allow REPL to load resource definitions."""
    pass
