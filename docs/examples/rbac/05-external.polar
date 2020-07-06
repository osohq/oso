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

role(actor: User, "employee") if
    actor.name = "alice"
    | actor.name = "bhavik"
    | actor.name = "cora"
    | role(actor, "accountant");

role(actor: User, "accountant") if
    actor.name = "deirdre" 
    | actor.name = "ebrahim" 
    | actor.name = "frantz"
    | role(actor, "admin");

role(actor: User, "admin") if
    actor.name = "greta" 
    | actor.name = "han"
    | actor.name = "iqbal";






role(actor: User, "employee") if
    actor.role = "employee";

role(actor: User, "accountant") if
    actor.role = "accountant";

role(actor: User, "admin") if
    actor.role = "admin";

# Employees can submit expenses
allow(actor, "submit", "expense") if
    role(actor, "employee");

# Accountants can view expenses
allow(actor, "view", "expense") if
    role(actor, "accountant");

# Admins can approve expenses
allow(actor, "approve", "expense") if
    role(actor, "admin");
