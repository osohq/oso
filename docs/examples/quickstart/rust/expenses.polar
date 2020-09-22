allow(actor: String, "GET", expense: Expense) if
    actor.ends_with("@example.com");


# allow(actor: String, "GET", expense: Expense) if
#     expense.submitted_by = actor;
