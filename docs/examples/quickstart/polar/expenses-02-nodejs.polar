allow(actor: String, "GET", expense: Expense) if
    expense.submittedBy = actor;
