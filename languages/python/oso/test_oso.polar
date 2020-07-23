allow(actor, action, resource) if
    allowRole(role, action, resource) and
    actorInRole(actor, role, resource);

allow(token, action, resource) if
    jwt = new Jwt{token: token} and
    jwt.attributes = attributes and
    allow(attributes, action, resource);

allow(_: {sub: sub}, action, resource) if
    allow(new Actor{name: sub}, action, resource);

allow("guest", action, resource) if
    allow(new Actor{name: "guest"}, action, resource);

allow(_: {username: name}, action, resource) if
    allow(new Actor{name: name}, action, resource);

allow(actor: Actor, "get", resource: Widget);
allow(actor: Actor, "create", resource: Company) if
    resource.role(actor) = "admin";

allow(actor: Actor, "frob", resource: Company) if
    actor.company.id = resource.id;

allow(actor: Actor, "list", Company) if
    actor.name = "auditor";
