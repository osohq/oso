"""
Example application integrated with Oso.

The application does not currently do much except define application
data structures that might already exist in, for example, a database.
"""

# External imports
from oso import Oso, polar_class

# Stdlib imports
import os
from pathlib import Path
import sys

# Local imports
from data import *


@polar_class
class User:
    """User model"""

    # username
    name: str
    # global role
    role: str
    # user's location
    location: str

    def __init__(self, name="", role="", location=""):
        self.name = name
        self.role = role
        self.location = location

    @classmethod
    def by_name(cls, name=""):
        """Lookup method to get a `User` object from the string name"""
        if name in USERS:
            return User(name, **USERS[name])
        else:
            # empty/non-existing user
            return User()

    def employees(self):
        """Returns the employees managed by this user"""
        if self.name in MANAGERS:
            for name in MANAGERS[self.name]:
                yield User.by_name(name)


@polar_class
class Expense:
    """Expense model"""

    def __init__(self, amount: int, submitted_by: str, location: str, project_id: int):
        self.amount = amount
        self.submitted_by = submitted_by
        self.location = location
        self.project_id = project_id

    @classmethod
    def id(cls, id: int):
        if id < len(EXPENSES):
            return Expense(**EXPENSES[id])
        else:
            return Expense()


@polar_class
class Project:
    """Project model"""

    def __init__(self, team_id: int):
        self.team_id = team_id

    @classmethod
    def id(cls, id: int):
        if id < len(PROJECTS):
            return Project(**PROJECTS[id])
        else:
            return Project()


@polar_class
class Team:
    """Team model"""

    def __init__(self, organization_id: int):
        self.organization_id = organization_id

    @classmethod
    def id(cls, id: int):
        if id < len(TEAMS):
            return Team(**TEAMS[id])
        else:
            return Team()


@polar_class
class Organization:
    """Organization model"""

    def __init__(self, name: str):
        self.name = name

    @classmethod
    def id(cls, id: int):
        if id < len(ORGANIZATIONS):
            return Organization(**ORGANIZATIONS[id])
        else:
            return Organization()


@polar_class
class Env:
    """Helper class for Oso, looks up environment variables"""

    @classmethod
    def var(cls, variable):
        return os.environ.get(variable, None)


def load_oso():
    """Loads and returns the Oso policy"""
    oso = Oso()
    policy_path = Path(__file__).resolve().parent.parent / "expenses"
    oso.load_files(
        [
            # Policy Data
            policy_path / "data.polar",
            # Role definitions
            policy_path / "roles.polar",
            # ABAC policy
            policy_path / "abac.polar",
        ]
    )
    return oso


if __name__ == "__main__":
    """Loads and checks the policy.

    Run example with `python app.py repl` to run the REPL after loading
    the policy.
    """
    oso = load_oso()
    print("Policy loaded OK")

    if len(sys.argv) > 1 and sys.argv[1] == "repl":
        oso.repl()
