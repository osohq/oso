group PatientData(Lab, Order, Test);

allow(user: User, "read", resource: PatientData) if
    user.role = "medical_staff" and
    user.treated(resource.patient);
