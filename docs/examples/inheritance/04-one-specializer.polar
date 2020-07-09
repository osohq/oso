allow(actor: Actor, "read", resource: PatientData) if
    actor.role = "medical_staff",
    actor.treated(resource.patient) = true;
