allow(actor, action, resource) :=
    allowRole(role, action, resource),
    actorInRole(actor, role, resource);

allow(token, action, resource) :=
    jwt = Jwt{token: token},
    jwt.attributes = attributes,
    allow(attributes, action, resource);

allow({sub: sub}, action, resource) :=
    allow(Actor{name: sub}, action, resource);

allow("guest", action, resource) :=
    allow(Actor{name: "guest"}, action, resource);

allow({username: name}, action, resource) :=
    allow(Actor{name: name}, action, resource);

allow(actor: Actor, "get", resource: Widget);
allow(actor: Actor, "create", resource: Company) :=
    resource.role(actor) = "admin";

allow(actor: Actor, "frob", resource: Company) :=
    actor.company.id = resource.id;
