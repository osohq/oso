# Hierarchies

## NEW CONCEPTS:
# - recursive attributes
# - representing hierachies

allow(actor: User, "view", resource: Expense) if
    employee = actor.employees and
    employee.name = resource.submittedBy;

# start-manages-rule
allow(actor: User, "view", resource: Expense) if
    manages(actor, employee) and
    employee.name = resource.submittedBy;
# end-manages-rule

# start-hierarchy-rule
# Management hierarchies
manages(manager: User, employee) if
    employee = manager.employees()
    or manages(manager.employees(), employee);


# Now Cora can view the expense because Cora manages Bhavik who manages Alice
?= allow(new User("cora"), "view", Expense.id(0));
# end-hierarchy-rule
