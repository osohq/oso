# RBAC + ABAC

## NEW CONCEPTS:
# - conditional roles
# - resource-specific roles

# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) :=
    role(actor, "accountant"),
    actor.location = resource.location;

# Alice is an admin of Project 1
projectRole("alice", "admin", project: Project) :=
    project.id = 1;

# Project admins can view expenses of the project
allow(actor: User, "view", resource: Expense) :=
    projectRole(actor, "admin", resource.project);

# Alice is an employee of ACME
orgRole(actor: User, "employee", organization: Organization) :=
    actor.name = "alice", organization.name = "ACME";

# employees can be team guests on a team
teamRole(actor: User, "guest", team: Team) :=
    orgRole(actor, "employee", team.organization);

# team guests are project guests
projectRole(actor: User, "guest", project: Project) :=
    teamRole(actor, "guest", project.team);


# This is not great
allow(actor: User, "view", resource: Expense) :=
    orgRole(actor, Role { name: "admin", organization: resource.organization }) |
    teamRole(actor, Role { name: "admin", team: resource.team }) |
    projectRole(actor, Role { name: "admin", project: resource.project });
