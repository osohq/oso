can_read_patient_data(actor, "read", resource) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient) = true;

## START MARKER ##
allow(actor: Actor, "read", resource: Order) if
    can_read_patient_data(actor, "read", resource);

allow(actor: Actor, "read", resource: Test) if
    can_read_patient_data(actor, "read", resource);

allow(actor: Actor, "read", resource: Lab) if
    can_read_patient_data(actor, "read", resource);
