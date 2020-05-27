allow(actor, action, resource) :=
    actorInRole(actor, role, resource),
    allowRole(role, action, resource);
