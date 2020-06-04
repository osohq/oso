# Bhavik is an admin at ACME
role(actor: User, "admin", organization: Organization) :=
    actor.name = "bhavik", organization.name = "ACME";
