# Hierarchies

## NEW CONCEPTS:
# - recursive attributes
# - representing hierachies

allow(actor: User, "view", resource: Expense) if
    employee = actor.employees,
    employee.name = resource.submitted_by;

# start-manages-rule
allow(actor: User, "view", resource: Expense) if
    manages(actor, employee),
    employee.name = resource.submitted_by;
# end-manages-rule

# start-hierarchy-rule
# Management hierarchies
manages(manager: User, employee) if
    employee = manager.employees() |
    manages(manager.employees(), employee);


# Now Cora can view the expense because Cora manages Bhavik who manages Alice
?= allow(new User { name: "cora"}, "view", new Expense { id: 0 });
# end-hierarchy-rule
