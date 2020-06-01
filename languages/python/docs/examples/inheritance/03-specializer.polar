can_read_patient_data(actor, "read", resource) :=
    actor.role = "medical_staff",
    actor.treated(resource.patient) = true;

## START MARKER ##
allow(actor: Actor, "read", resource: Order) :=
    can_read_patient_data(actor, "read", resource);

allow(actor: Actor, "read", resource: Test) :=
    can_read_patient_data(actor, "read", resource);

allow(actor: Actor, "read", resource: Lab) :=
    can_read_patient_data(actor, "read", resource);
