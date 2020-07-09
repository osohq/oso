# Investors have read access to startups
allow(actor, "read", company: StartUp) if
    cut() and
    actor = company.investors;
