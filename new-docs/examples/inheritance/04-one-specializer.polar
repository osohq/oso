allow(actor: Actor, "read", resource: PatientData) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient);
