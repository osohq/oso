# Top-level rules

allow(_user, "GET", http_request) if
    http_request.path = "/";

allow(_user: User, "GET", http_request) if
    http_request.path = "/whoami";

# Allow by path segment
allow(user, action, http_request) if
    http_request.path.split("/") = [_, stem, *rest]
    and allow(user, action, stem, rest);

### Expenses rules

# by HTTP method
allow(_user, "GET", "expense", _rest);
allow(_user, "PUT", "expense", "submit");

allow(user: User, "read", expense) if
    submitted(user, expense);

allow(user, "create", expense: Expense) if
    submitted(user, expense);

submitted(user: User, expense: Expense) if
    user.id = expense.user_id;

### Organization rules
allow(user: User, _action, "organization", organization_id) if
    user.organization_id = organization_id;
