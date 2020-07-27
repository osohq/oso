allow(actor, "GET", expense) if
    expense.submitted_by = actor;
