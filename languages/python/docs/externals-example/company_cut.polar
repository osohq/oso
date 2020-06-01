# Investors have read access to startups
allow(actor, "read", company: StartUp) :=
    cut(),
    actor = company.investors;