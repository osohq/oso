# company-start
from oso import polar_class, Oso

NAMES = {
    1: "company-one",
    2: "company-two",
}

MEMBERS = {
    1: ["dhatch", "sam"],
    2: ["leina"],
}

DEPARTMENT_MEMBERS = {
    1: {"engineering": ["dhatch"], "executive": ["sam"]},
    2: {"engineering": ["leina"], "executive": []},
}

@polar_class
class Company:
    def __init__(self, id=None, default_role=""):
        self.id = id
        self.default_role = default_role

    def name(self):
        return NAMES[self.id]

    def members(self):
        yield from MEMBERS[self.id]

    def department_members(self, department_name):
        yield from DEPARTMENT_MEMBERS[self.id][department_name]
# company-end

# startup-start
INVESTORS = {
    1: ["chris", "peter", "ron"],
    2: ["reid", "dave", "fred"],
}

@polar_class
class StartUp(Company):

    def investors(self):
        yield from INVESTORS[self.id]
# startup-end

def load(kb):
    """Allow REPL to load resource definitions."""
    pass
