can_read_patient_data(actor, "read", resource) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient);

allow(actor: User, "read", resource) if
    can_read_patient_data(actor, "read", resource);
