# Simple RBAC

## NEW CONCEPTS:
# - basic allow rules with variables
# - defining the custom `role` predicate

role(actor, "employee") if
    actor = "alice" 
    | actor = "bhavik" 
    | actor = "cora";

role(actor, "accountant") if
    actor = "deirdre" 
    | actor = "ebrahim" 
    | actor = "frantz";

role(actor, "admin") if
    actor = "greta" 
    | actor = "han" 
    | actor = "iqbal";

# Employees can submit expenses
allow(actor, "submit", "expense") if
    role(actor, "employee");

# Accountants can view expenses
allow(actor, "view", "expense") if
    role(actor, "accountant");

# Admins can approve expenses
allow(actor, "approve", "expense") if
    role(actor, "admin");

# Deirdre the accountant can view expenses
?= allow("deirdre", "view", "expense");

# but cannot submit or approve them
?= !allow("deirdre", "submit", "expense");
?= !allow("deirdre", "approve", "expense");
