allow(actor: String, "GET", _expense: Expense) if
    actor.endsWith("@example.com");
