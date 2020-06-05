# Hierarchies

## NEW CONCEPTS:
# - recursive attributes
# - representing hierachies

allow(actor: User, "view", resource: Expense) :=
    employee = actor.employees,
    employee.name = resource.submitted_by;

# Management hierarchies
allow(actor: User, "view", resource: Expense) :=
    manages(actor, employee),
    isa(employee, User { name: resource.submitted_by });

manages(manager: User, employee) :=
    employee = manager.employees() |
    manages(manager.employees(), employee);

# Now Cora can view the expense because Cora manager Bhavik who manager Alice
?= allow(User { name: "cora"}, "view", Expense { id: 0 });
