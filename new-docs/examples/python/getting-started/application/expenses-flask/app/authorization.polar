# Top-level rules

allow(_user, "GET", request: Request) if
    request.path = "/";

allow(_user: User, "GET", request: Request) if
    request.path = "/whoami";

# Allow by path segment
allow(user, action, request: Request) if
    request.path.split("/") = [_, stem, *rest]
    and allow_by_path(user, action, stem, rest);

### Expense rules

# by HTTP method
allow_by_path(_user, "GET", "expenses", _rest);
allow_by_path(_user, "PUT", "expenses", ["submit"]);

# by model
allow(user: User, "read", expense: Expense) if
    submitted(user, expense);

submitted(user: User, expense: Expense) if
    user.id = expense.user_id;

### Organization rules
allow_by_path(_user, "GET", "organizations", _rest);
allow(user: User, "read", organization: Organization) if
    user.organization_id = organization.id;
