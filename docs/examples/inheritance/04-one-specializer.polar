allow(actor: Actor, "read", resource: PatientData) :=
    actor.role = "medical_staff",
    actor.treated(resource.patient) = true;
