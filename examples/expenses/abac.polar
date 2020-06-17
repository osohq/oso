# Users can view expenses they submitted
allow(actor: User, "view", resource: Expense) :=
    resource.submitted_by = actor.name;

?= allow(User { name: "alice"}, "view", Expense {  id: 0});

# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) :=
    role(actor, "accountant"),
    actor.location = resource.location;

# As an accountant, deirdre can view expenses in the same location
?= allow(User { name: "deirdre"}, "view", Expense { id: 0 });

### RBAC Hierarchy
# Expense > Project > Team > Organization

# Project admins can view expenses of the project
allow(actor: User, "view", resource: Expense) :=
    role(actor, "admin", Project { id: resource.project_id });

# Project roles inherit from Team roles
role(actor: User, role, project: Project) :=
    role(actor, role, Team { id: project.team_id });

# Team roles inherit from Organization roles
role(actor: User, role, team: Team) :=
    role(actor, role, Organization { id: team.organization_id });


# As an admin of ACME, Bhavik can view expenses in the org
?= allow(User { name: "bhavik" }, "view", Expense { id: 0 });


# Management hierarchies
allow(actor: User, "view", resource: Expense) :=
    manages(actor, employee),
    isa(employee, User { name: resource.submitted_by });

manages(manager: User, employee) :=
    employee = manager.employees() |
    manages(manager.employees(), employee);

# Cora can view the expense because Cora manager Bhavik who manager Alice
?= allow(User { name: "cora"}, "view", Expense { id: 0 });

# If ENV="development" is set as an environment variable
# Then allow all
allow(_user, _action, _resource) :=
    Env{}.var("ENV") = "development";
