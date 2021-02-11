# Simple RBAC

## NEW CONCEPTS:
# - basic allow rules with variables
# - defining the custom `role` predicate

role(actor: String, "employee") if
    actor = "alice" or
    actor = "bhavik" or
    actor = "cora";

role(actor: String, "accountant") if
    actor = "deirdre" or
    actor = "ebrahim" or
    actor = "frantz";

role(actor: String, "admin") if
    actor = "greta" or
    actor = "han" or
    actor = "iqbal";

# Employees can submit expenses
allow(actor: String, "submit", "expense") if
    role(actor, "employee");

# Accountants can view expenses
allow(actor: String, "view", "expense") if
    role(actor, "accountant");

# Admins can approve expenses
allow(actor: String, "approve", "expense") if
    role(actor, "admin");

# Deirdre the accountant can view expenses
?= allow("deirdre", "view", "expense");

# but cannot submit or approve them
?= not allow("deirdre", "submit", "expense");
?= not allow("deirdre", "approve", "expense");
