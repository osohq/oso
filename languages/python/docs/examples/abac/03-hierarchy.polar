# Hierarchies

## NEW CONCEPTS:
# - recursive attributes
# - representing hierachies

allow(actor: User, "view", resource: Expense) :=
    employee = actor.employees,
    employee.name = resource.submitted_by;

# start-manages-rule
allow(actor: User, "view", resource: Expense) :=
    manages(actor, employee),
    employee.name = resource.submitted_by;
# end-manages-rule

# start-hierarchy-rule
# Management hierarchies
manages(manager: User, employee) :=
    employee = manager.employees() |
    manages(manager.employees(), employee);


# Now Cora can view the expense because Cora manages Bhavik who manages Alice
?= allow(User { name: "cora"}, "view", Expense { id: 0 });
# end-hierarchy-rule
