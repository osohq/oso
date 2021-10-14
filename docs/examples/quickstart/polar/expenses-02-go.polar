allow(actor: String, "GET", expense: Expense) if
    expense.SubmittedBy = actor;
