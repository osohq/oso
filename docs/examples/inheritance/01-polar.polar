allow(actor: Actor, "read", resource: Order) :=
    actor.role = "medical_staff",
    actor.treated(resource.patient) = true;

allow(actor: Actor, "read", resource: Test) :=
    actor.role = "medical_staff",
    actor.treated(resource.patient) = true;

allow(actor: Actor, "read", resource: Lab) :=
    actor.role = "medical_staff",
    actor.treated(resource.patient) = true;
