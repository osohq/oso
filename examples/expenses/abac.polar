# Users can view expenses they submitted
allow(actor: User, "view", resource: Expense) :=
    resource.submitted_by = actor.name;

?= allow(new User { name: "alice"}, "view", new Expense {  id: 0});

# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) :=
    role(actor, "accountant"),
    actor.location = resource.location;

# As an accountant, deirdre can view expenses in the same location
?= allow(new User { name: "deirdre"}, "view", new Expense { id: 0 });

### RBAC Hierarchy
# Expense > Project > Team > Organization

# Project admins can view expenses of the project
allow(actor: User, "view", resource: Expense) :=
    role(actor, "admin", new Project { id: resource.project_id });

# Project roles inherit from Team roles
role(actor: User, role, project: Project) :=
    role(actor, role, new Team { id: project.team_id });

# Team roles inherit from Organization roles
role(actor: User, role, team: Team) :=
    role(actor, role, new Organization { id: team.organization_id });


# As an admin of ACME, Bhavik can view expenses in the org
?= allow(new User { name: "bhavik" }, "view", new Expense { id: 0 });


# Management hierarchies
allow(actor: User, "view", resource: Expense) :=
    manages(actor, employee),
    employee isa User { name: resource.submitted_by };

manages(manager: User, employee) :=
    employee = manager.employees() |
    manages(manager.employees(), employee);

# Now Cora can view the expense because Cora manager Bhavik who manager Alice
?= allow(new User { name: "cora"}, "view", new Expense { id: 0 });

# If ENV="development" is set as an environment variable
# Then allow all
allow(_user, _action, _resource) :=
    new Env{}.var("ENV") = "development";
