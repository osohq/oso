can_read_patient_data(actor, "read", resource) :=
    actor.role = "medical_staff",
    actor.treated(resource.patient) = true;

allow(actor: Actor, "read", resource) :=
    can_read_patient_data(actor, "read", resource);
