from random import choice, randint

role = "role"
location = "location"

LOC = ["NYC", "London", "Berlin"]

USERS = {
    "alice": {role: "employee", location: LOC[0]},
    "bhavik": {role: "employee", location: LOC[1]},
    "cora": {role: "employee", location: LOC[2]},
    "deirdre": {role: "accountant", location: LOC[0]},
    "ebrahim": {role: "accountant", location: LOC[1]},
    "frantz": {role: "accountant", location: LOC[2]},
    "greta": {role: "admin", location: LOC[0]},
    "han": {role: "admin", location: LOC[1]},
    "iqbal": {role: "admin", location: LOC[2]},
}

MANAGERS = {
    "cora": ["bhavik"],
    "bhavik": ["alice"],
}

PROJECTS = [
    {"team_id": 0},
    {"team_id": 0},
    {"team_id": 1},
    {"team_id": 2},
]

TEAMS = [
    {"organization_id": 0},
    {"organization_id": 0},
    {"organization_id": 1},
]

ORGANIZATIONS = [
    {"name": "ACME"},
    {"name": "Bancroft Industries"},
]

EXPENSES = [
    {"submitted_by": "alice", "amount": 500, "location": "NYC", "project_id": 2}
]


def generate_expenses(lines=50):
    for _ in range(lines):
        # what = choice(WORDS)
        who = choice(list(USERS.keys()))
        amount = randint(100, 10000)
        where = choice(USERS[who][location])
        project = randint(0, len(PROJECTS))
        EXPENSES.append(
            {
                "amount": amount,
                "submitted_by": who,
                location: where,
                "project_id": project,
            }
        )


generate_expenses()
