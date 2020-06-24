blocked(actor) := actor.name in ["Mallory", "Wallace"];
allowed(actor) := actor.name in ["Alice"];

ordered(actor, action, resource: ComplicatedResource, result) :=
    # Deny if actor is blocked
    (blocked(actor), result = "deny") |
    # Allow if actor is superuser
    (actor.role = "superuser", result = "allow") |
    # Allow if actor is in allow list.
    (allowed(actor), result = "allow") |
    # Allow if resource is unrestricted.
    (resource.unrestricted = true, result = "allow") |
    # Default deny
    result = "deny";


allow(actor, action, resource) :=
    # Use the ordered rule to find an allow result.
    # cut causes eval to stop on first allow result
    ordered(actor, action, resource, result), cut(), result = "allow";

ordered2(actor, action, resource: ComplicatedResource) :=
    !blocked(actor),
    # Allow if actor is superuser
    (actor.role = "superuser") |
    (allowed(actor)) |
    (resource.unrestricted = true);


allow(actor, action, resource) :=
    # Use the ordered rule to find an allow result.
    # cut causes eval to stop on first allow result
    ordered(actor, action, resource, result), cut(), result = "allow";

allow2(actor, action, resource) :=
    # Use the ordered rule to find an allow result.
    # cut causes eval to stop on first allow result
    ordered2(actor, action, resource);
