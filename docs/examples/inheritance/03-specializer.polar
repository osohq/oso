can_read_patient_data(actor, "read", resource) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient);

## START MARKER ##
allow(actor: User, "read", resource: Order) if
    can_read_patient_data(actor, "read", resource);

allow(actor: User, "read", resource: Test) if
    can_read_patient_data(actor, "read", resource);

allow(actor: User, "read", resource: Lab) if
    can_read_patient_data(actor, "read", resource);
