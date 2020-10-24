# Users can view expenses they submitted
allow(actor: User, "view", resource: Expense) if
    resource.submitted_by = actor.name;

?= allow(User.by_name("alice"), "view", Expense.id(0));

# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) if
    role(actor, "accountant") and
    actor.location = resource.location;

# As an accountant, deirdre can view expenses in the same location
?= allow(User.by_name("deirdre"), "view", Expense.id(0));

### RBAC Hierarchy
# Expense > Project > Team > Organization

# Project admins can view expenses of the project
allow(actor: User, "view", resource: Expense) if
    role(actor, "admin", Project.id(resource.project_id));

# Project roles inherit from Team roles
role(actor: User, role, project: Project) if
    role(actor, role, Team.id(project.team_id));

# Team roles inherit from Organization roles
role(actor: User, role, team: Team) if
    role(actor, role, Organization.id(team.organization_id));


# As an admin of ACME, Bhavik can view expenses in the org
?= allow(User.by_name("bhavik"), "view", Expense.id(0));


# Management hierarchies
allow(actor: User, "view", resource: Expense) if
    manages(actor, employee) and
    employee matches User{ name: resource.submitted_by };

manages(manager: User, employee) if
    report in manager.employees() and
    report = employee or
    manages(report, employee);

# Now Cora can view the expense because Cora manager Bhavik who manager Alice
?= allow(User.by_name("cora"), "view", Expense.id(0));

# If the environment variable ENV = "development" then allow all
allow(_user, _action, _resource) if
    Env.var("ENV") = "development";
