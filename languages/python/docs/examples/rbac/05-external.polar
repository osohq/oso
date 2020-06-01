# complex RBAC (using external data)

## NEW CONCEPTS:
# - external classes and methods
# - rule specializers

# Python:
# 
# @polar_class
# class User:
#   def role(self):
#       return lookup_role(self.id)

role(actor: User, "employee") :=
    actor.name = "alice"
    | actor.name = "bhavik"
    | actor.name = "cora"
    | role(actor, "accountant");

role(actor: User, "accountant") :=
    actor.name = "deirdre" 
    | actor.name = "ebrahim" 
    | actor.name = "frantz"
    | role(actor, "admin");

role(actor: User, "admin") :=
    actor.name = "greta" 
    | actor.name = "han"
    | actor.name = "iqbal";






role(actor: User, "employee") :=
    actor.role = "employee";

role(actor: User, "accountant") :=
    actor.role = "accountant";

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
