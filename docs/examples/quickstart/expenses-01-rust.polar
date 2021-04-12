allow(actor: String, "GET", _expense: Expense) if
    actor.ends_with("@example.com");
