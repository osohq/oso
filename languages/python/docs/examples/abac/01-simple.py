from oso import polar_class

EXPENSES = [
    {"submitted_by": "alice", "amount": 500, "location": "NYC", "project_id": 2}
]

# expense-class-start
@polar_class(from_polar="by_id")
class Expense:
    """Expense model"""  # expense-class-end

    def __init__(self, amount: int, submitted_by: str, location: str, project_id: int):
        self.amount = amount
        self.submitted_by = submitted_by
        self.location = location
        self.project_id = project_id

    @classmethod
    def by_id(cls, id: int):
        if id < len(EXPENSES):
            return Expense(**EXPENSES[id])
        else:
            return Expense()


MANAGERS = {
    "cora": ["bhavik"],
    "bhavik": ["alice"],
}

# user-class-start
@polar_class
class User:
    def __init__(self, name, location: str = None):
        self.name = name  # user-class-end
        self.location = location or "NYC"

    def employees(self):
        """Returns the employees managed by this user"""
        if self.name in MANAGERS:
            for name in MANAGERS[self.name]:
                yield User(name)


@polar_class(from_polar="by_id")
class Project:
    """Project model"""

    def __init__(self, id: int, team_id: int):
        self.id = id
        self.team_id = team_id

    @classmethod
    def by_id(cls, id: int):
        return Project(id, 0)


@polar_class(from_polar="by_id")
class Team:
    """Team model"""

    def __init__(self, organization_id: int):
        self.organization_id = organization_id

    @classmethod
    def by_id(cls, id: int):
        return Team(0)


@polar_class(from_polar="by_id")
class Organization:
    """Organization model"""

    def __init__(self, name: str):
        self.name = name

    @classmethod
    def by_id(cls, id: int):
        return Organization("ACME")
