allow(actor: User, "read", resource: Order) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient);

allow(actor: User, "read", resource: Test) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient);

allow(actor: User, "read", resource: Lab) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient);
