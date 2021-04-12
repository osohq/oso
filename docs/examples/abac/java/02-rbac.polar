# RBAC + ABAC

## NEW CONCEPTS:
# - conditional roles
# - resource-specific roles

# simple-rule-start
# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) if
    role(actor, "accountant") and
    actor.location = resource.location;
# simple-rule-end

role(_: User {name: "deirdre"}, "accountant");

# As an accountant, deirdre can view expenses in the same location
?= allow(new User("deirdre"), "view", Expense.id(0));

### RBAC Hierarchy
# Expense > Project > Team > Organization

# project-rule-start
# Alice is an admin of Project 1
role(_: User { name: "alice" }, "admin", _: Project { id: 1 });

# Project admins can view expenses of the project
allow(actor: User, "view", resource: Expense) if
    role(actor, "admin", Project.id(resource.projectId));
# project-rule-end

# role-inherit-start
# Bhavik is an admin of ACME
role(_: User { name: "bhavik" }, "admin",  _: Organization { name: "ACME" });

# Team roles inherit from Organization roles
role(actor: User, role: String, team: Team) if
    role(actor, role, Organization.id(team.organizationId));

# Project roles inherit from Team roles
role(actor: User, role: String, project: Project) if
    role(actor, role, Team.id(project.teamId));
# role-inherit-end

# As an admin of ACME, Bhavik can view expenses in the org
?= allow(new User("bhavik"), "view", Expense.id(0));
?= not allow(new User("cora"), "view", Expense.id(0));
