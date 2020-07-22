allow(actor, "GET", _expense) if
    actor.end_with?("@example.com");
