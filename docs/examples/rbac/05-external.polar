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
    actor.name = "alice" or
    actor.name = "bhavik" or
    actor.name = "cora" or
    role(actor, "accountant");

role(actor: User, "accountant") if
    actor.name = "deirdre" or
    actor.name = "ebrahim" or
    actor.name = "frantz" or
    role(actor, "admin");

role(actor: User, "admin") if
    actor.name = "greta" or
    actor.name = "han" or
    actor.name = "iqbal";






role(actor: User, "employee") if
    actor.role = "employee";

role(actor: User, "accountant") if
    actor.role = "accountant";

role(actor: User, "admin") if
    actor.role = "admin";

# Employees can submit expenses
allow(actor: User, "submit", "expense") if
    role(actor, "employee");

# Accountants can view expenses
allow(actor: User, "view", "expense") if
    role(actor, "accountant");

# Admins can approve expenses
allow(actor: User, "approve", "expense") if
    role(actor, "admin");
