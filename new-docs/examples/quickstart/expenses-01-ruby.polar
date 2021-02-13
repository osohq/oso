allow(actor: String, "GET", _expense: Expense) if
    actor.end_with?("@example.com");
