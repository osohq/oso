allow(actor, action, resource) if
    allowRole(role, action, resource) and
    actorInRole(actor, role, resource);

allow(_: {sub: sub}, action, resource) if
    allow(new User(sub), action, resource);

allow("guest", action, resource) if
    allow(new User("guest"), action, resource);

allow(_: {username: name}, action, resource) if
    allow(new User(name), action, resource);

allow(_actor: User, "get", _resource: Widget);
allow(actor: User, "create", resource: Company) if
    resource.role(actor) = "admin";

allow(actor: User, "frob", resource: Company) if
    resource in actor.companies();

allow(actor: User, "list", Company) if
   actor.name = "auditor";
