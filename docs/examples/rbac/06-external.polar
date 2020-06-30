role(actor: User, "employee") :=
    actor.role = "employee"
    | role(actor, "accountant");

role(actor: User, "accountant") :=
    actor.role = "accountant"
    | role(actor, "admin");

role(actor: User, "admin") :=
    actor.role = "admin";

# Employees can submit expenses
allow(actor, "submit", "expense") :=
    role(actor, "employee");

# Accountants can view expenses
allow(actor, "view", "expense") :=
    role(actor, "accountant");

# Admins can approve expenses
allow(actor, "approve", "expense") :=
    role(actor, "admin");
