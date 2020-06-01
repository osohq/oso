# Hierarchies

## NEW CONCEPTS:
# - recursive attributes
# - representing hierachies

allow(actor: User, "view", resource: Expense) :=
    employee = actor.employees,
    employee.name = resource.submitted_by;

allow(actor: User, "view", resource: Expense) :=
    manages(actor, employee),
    employee.name = resource.submitted_by;

manages(manager: User, employee: User) :=
    employee = manager.employees;

manages(manager: User, employee: User) :=
    employee = manager.employees |
    manages(manager.employees, employee);
