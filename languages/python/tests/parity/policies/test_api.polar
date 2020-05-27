allow(actor, action, resource) :=
    actorInRole(actor, role, resource),
    allowRole(role, action, resource);

actorInRole(actor, role, resource: Widget) :=
    role = resource.company.role(actor);

allow(actor, "get", Http{path: path}) :=
    PathMapper{template: "/widget/{id}"}.map(path) = {id: id},
    allow(actor, "get", Widget{id: id});

allow(actor, "post", Http{path: path}) :=
    PathMapper{template: "/widget/"}.map(path) = {},
    allow(actor, "create", Widget{});

allow(actor, "what", Http{path: path}) :=
    PathMapper{template: "/widget/{id}"}.map(path) = {id: id},
    allow(actor, "unparameterised_get", Widget{id: id});

allow(actor, "what", Http{path: path, query: {param: "foo"}}) :=
    PathMapper{template: "/widget/{id}"}.map(path) = {id: id},
    allow(actor, "parameterised_get", Widget{id: id});

allow(actor, "get", resource: Widget) := resource.frob("Widget") = x;
allow(actor, "get", resource: DooDad) := resource.frob("DooDad") = x;

# Frobbing a Widget writes an entry into a global frobbed list,
# which can then be checked to ensure correct method ordering.
# See test_allow, test_method_resolution_order, test_cut.
allow_with_cut(actor, "get", resource: Widget) := resource.frob("Widget") = x, cut();
allow_with_cut(actor, "get", resource: DooDad) := resource.frob("DooDad") = x, cut();

allowRole("admin", "create", resource: Widget);

allow(actor: Actor, "frob", resource: Widget) :=
    actor.company.id = resource.company.id,
    actor.company.default_role = resource.company.default_role,
    actor.company.roles = resource.company.roles;

# for testing resource mappings with query parameters
allow(actor, "parameterised_get", resource: Widget) :=
    resource.id = "12";

# When choosing which `checkResource` is more specific, will compare
# the two unifiers (`resource: Widget` in both cases) against the input
# argument.
#
# The `is_subspecializer` check compares the application class of `resource`
# This test checks that works okay.
allow_two(actor, action, resource) := checkResource(_x, resource);
checkResource(1, resource: Widget); # two slightly different specs so need to check
checkResource("1", resource: Widget); # which to prioritise

?= allow_two(_actor, _action, Widget{});

# for testing lists
allow(actor: Actor, "invite", resource: Widget) :=
    actor.group = "social";

allow(actor: Actor, "keep", resource: Widget) :=
    actor.widget.id = resource.id,
    actor.widget.name = resource.name;

# for testing iter
allow(actor: Actor, "can_have", Widget {name: "stapler"}) :=
    isa(actor.companies_iter, Company {id: "Initech"});
