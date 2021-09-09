allow(actor, action, resource) if
    actorInRole(actor, role, resource) and
    allowRole(role, action, resource);

actorInRole(actor, role, resource: Widget) if
    role = resource.company().role(actor);

allow(actor, "get", _: Http{path: path}) if
    new PathMapper("/widget/{id}").map(path) = {id: id} and
    allow(actor, "get", new Widget(id: id));

allow(actor, "post", _: Http{path: path}) if
    new PathMapper(template: "/widget/").map(path) = {} and
    allow(actor, "create", new Widget());

allow(actor, "what", _: Http{path: path}) if
    new PathMapper("/widget/{id}").map(path) = {id: id} and
    allow(actor, "unparameterised_get", new Widget(id: id));

allow(actor, "what", _: Http{path: path, query: {param: "foo"}}) if
    new PathMapper("/widget/{id}").map(path) = {id: id} and
    allow(actor, "parameterised_get", new Widget(id: id));

allow(_actor, "get", resource: Widget) if resource.frob("Widget") = _x;
allow(_actor, "get", resource: DooDad) if resource.frob("DooDad") = _x;

# Frobbing a Widget writes an entry into a global frobbed list,
# which can then be checked to ensure correct method ordering.
# See test_allow, test_method_resolution_order, test_cut.
allow_with_cut(_actor, "get", resource: Widget) if cut and resource.frob("Widget") = _x;
allow_with_cut(_actor, "get", resource: DooDad) if cut and resource.frob("DooDad") = _x;

allowRole("admin", "create", _resource: Widget);

allow(actor: User, "frob", resource: Widget) if
    resource.company() in actor.companies();

# for testing resource mappings with query parameters
allow(_actor, "parameterised_get", resource: Widget) if
    resource.id = "12";

# When choosing which `checkResource` is more specific, will compare
# the two unifiers (`resource: Widget` in both cases) against the input
# argument.
#
# The `is_subspecializer` check compares the application class of `resource`
# This test checks that works okay.
allow_two(_actor, _action, resource) if checkResource(_x, resource);
checkResource(1, _resource: Widget); # two slightly different specs so need to check
checkResource("1", _resource: Widget); # which to prioritise

?= allow_two(_actor, _action, new Widget());

# for testing lists
allow(actor: User, "invite", _resource: Widget) if
    "social" in actor.groups;

allow(actor: User, "keep", resource: Widget) if
    actor.widget = resource and
    actor.widget.name = resource.name;

# for testing iter
allow(actor: User, "can_have", _: Widget{name: "stapler"}) if
    company in actor.companies_iter() and
    company matches Company{id: "Initech"};

# test fails on not iterable iter
allow(actor: User, "tries_to_get", _) if
    1 in actor;
