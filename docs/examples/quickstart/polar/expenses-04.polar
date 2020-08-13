allow(actor: String, "GET", expense: Expense) if
    expense.submitted_by = actor;
