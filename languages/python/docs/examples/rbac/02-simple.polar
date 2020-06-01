# Simple RBAC with role inheritance (hierarchical)

## NEW CONCEPTS:
# - role hierarchy

# Accountants can do anything an employee can do
role(actor, "employee") :=
    actor = "alice"
    | actor = "bhavik"
    | actor = "cora"
    | role(actor, "accountant");

# Admins can do anything an accountant can do
role(actor, "accountant") :=
    actor = "deirdre" 
    | actor = "ebrahim" 
    | actor = "frantz"
    | role(actor, "admin");

role(actor, "admin") :=
    actor = "greta" 
    | actor = "han"
    | actor = "iqbal";

# Employees can submit expenses
allow(actor, "submit", "expense") :=
    role(actor, "employee");

# Accountants can view expenses
allow(actor, "view", "expense") :=
    role(actor, "accountant");

# Admins can approve expenses
allow(actor, "approve", "expense") :=
    role(actor, "admin");

# Deirdre the accountant can view and submit expenses
?= allow("deirdre", "view", "expense");
?= allow("deirdre", "submit", "expense");

# but cannot approve them
?= !allow("deirdre", "approve", "expense");

# Iqbal the administrator can do everything
?= allow("iqbal", "view", "expense");
?= allow("iqbal", "submit", "expense");
?= allow("iqbal", "approve", "expense");
