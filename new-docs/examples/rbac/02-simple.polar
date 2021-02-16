# Simple RBAC with role inheritance (hierarchical)

## NEW CONCEPTS:
# - role hierarchy

# accountant-inherits-from-employee-start
# Accountants can do anything an employee can do
role(actor, "employee") if
    actor = "alice" or
    actor = "bhavik" or
    actor = "cora" or
    role(actor, "accountant");
# accountant-inherits-from-employee-end

# admin-inherits-from-accountant-start
# Admins can do anything an accountant can do
role(actor, "accountant") if
    actor = "deirdre" or
    actor = "ebrahim" or
    actor = "frantz" or
    role(actor, "admin");
# admin-inherits-from-accountant-end

role(actor, "admin") if
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

# inline-queries-start
# Deirdre the accountant can view and submit expenses
?= allow("deirdre", "view", "expense");
?= allow("deirdre", "submit", "expense");

# but cannot approve them
?= not allow("deirdre", "approve", "expense");

# Iqbal the administrator can do everything
?= allow("iqbal", "view", "expense");
?= allow("iqbal", "submit", "expense");
?= allow("iqbal", "approve", "expense");
# inline-queries-end
