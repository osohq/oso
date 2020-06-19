# Assume actions are hierarchical: R < RW < RWC < RWCU
allow_model(actor, "read", resource) :=
    allow_model(actor, "R", resource);
allow_model(actor, "write", resource) :=
    allow_model(actor, "RW", resource);
allow_model(actor, "create", resource) :=
    allow_model(actor, "RWC", resource);
allow_model(actor, "unlink", resource) :=
    allow_model(actor, "RWCU", resource);

allow_model(actor, "R", resource) :=
    allow_model(actor, "RW", resource);
allow_model(actor, "RW", resource) :=
    allow_model(actor, "RWC", resource);
allow_model(actor, "RWC", resource) :=
    allow_model(actor, "RWCU", resource);

# Lookup role for user
role(user, role) := user.groups.id = role;

# Top-level
allow_model(actor, action, resource) :=
    allow_dhi_billing(actor, action, resource);

# Billing
allow_dhi_billing(actor, action, resource) :=
    role(actor, role),
    allow_dhi_billing_by_role(role, action, resource);

allow_dhi_billing_by_role("user_access.dhi_group_receptionist", "RWC", "dhi.bill");