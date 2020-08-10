allow(actor: String, "GET", _expense: String) if
    actor.endsWith("@example.com");
