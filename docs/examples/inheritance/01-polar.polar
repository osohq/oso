allow(actor: Actor, "read", resource: Order) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient) = true;

allow(actor: Actor, "read", resource: Test) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient) = true;

allow(actor: Actor, "read", resource: Lab) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient) = true;
