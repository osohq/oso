allow(actor, "GET", expense) if
    expense.submittedBy = actor;
