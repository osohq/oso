# RBAC + ABAC

## NEW CONCEPTS:
# - conditional roles
# - resource-specific roles

# simple-rule-start
# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) :=
    role(actor, "accountant"),
    actor.location = resource.location;
# simple-rule-end

role(User {name: "deirdre"}, "accountant");

# As an accountant, deirdre can view expenses in the same location
?= allow(new User { name: "deirdre" }, "view", new Expense { id: 0 });

### RBAC Hierarchy
# Expense > Project > Team > Organization

# project-rule-start
# Alice is an admin of Project 1
role(User { name: "alice" }, "admin", Project { id: 1 });

# Project admins can view expenses of the project
allow(actor: User, "view", resource: Expense) :=
    role(actor, "admin", new Project { id: resource.project_id });
# project-rule-end

# role-inherit-start
# Bhavik is an admin of ACME
role(User { name: "bhavik" }, "admin",  Organization { name: "ACME" });

# Team roles inherit from Organization roles
role(actor: User, role, team: Team) :=
    role(actor, role, new Organization { id: team.organization_id });

# Project roles inherit from Team roles
role(actor: User, role, project: Project) :=
    role(actor, role, new Team { id: project.team_id });
# role-inherit-end

# As an admin of ACME, Bhavik can view expenses in the org
?= allow(new User { name: "bhavik" }, "view", new Expense { id: 0 });
?= !allow(new User { name: "cora" }, "view", new Expense { id: 0 });
