role(actor: User, "employee") :=
    actor.role = "employee";
    | role(actor, "accountant");

role(actor: User, "accountant") :=
    actor.role = "accountant"
    | role(actor, "admin");

role(actor: User, "admin") :=
    actor.role = "admin";

# Employees can submit expenses
allow(actor, "submit", "expense") :=
    actor.role = "employee";

# Accountants can view expenses
allow(actor, "view", "expense") :=
    actor.role = "accountant";

# Admins can approve expenses
allow(actor, "approve", "expense") :=
    actor.role = "admin";
