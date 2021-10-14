allow(actor: String, "GET", _expense: Expense) if
    actor.EndsWith("@example.com");
