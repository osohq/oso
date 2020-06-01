actorInRole(actor, role, resource) :=
    role = resource.role(actor.username);

# Allow engineers to read company resources.
allowRole("engineering", "read", resource: Company);

# Allow execs to edit.
allowRole("executive", "edit", resource: Company);

?= allow(Actor{"username": "sam"}, "edit", Company{"id": 1});
?= allow(Actor{"username": "dhatch"}, "read", Company{"id": 1});
?= !(allow(Actor{"username": "dhatch"}, "edit", Company{"id": 1}));
