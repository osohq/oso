# roles-start
role(actor: User, "employee") if
    actor.role = "employee" or
    role(actor, "accountant");

role(actor: User, "accountant") if
    actor.role = "accountant" or
    role(actor, "admin");

role(actor: User, "admin") if
    actor.role = "admin";
# roles-end

# Employees can submit expenses
allow(actor: User, "submit", "expense") if
    role(actor, "employee");

# Accountants can view expenses
allow(actor: User, "view", "expense") if
    role(actor, "accountant");

# Admins can approve expenses
allow(actor: User, "approve", "expense") if
    role(actor, "admin");
