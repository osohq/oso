allow(_user, "GET", http_request) if
    http_request.path = "/";

allow(_user: User, "GET", http_request) if
    http_request.path = "/whoami";

allow(user: User, "GET", http_request) if
    http_request.path.split("/") = [_, *rest]
    and allow(user, http_request.method, rest);

allow(user: User, "GET", ["expenses", expense_id]) if
    user.id = Expense.lookup(expense_id).user_id.__str__();
