allow(actor: String, "GET", _expense: Expense) if
    actor.endswith("@example.com");
