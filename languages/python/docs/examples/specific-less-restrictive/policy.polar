allow(actor, action, resource: General) :=
    actor.num = 1 | actor.num = 2;

allow(actor, action, resource: Specific) := actor.num = 2;


_decide(actor, action, resource: General, result) :=
    cut(), actor.num = 1 | actor.num = 2;

_decide(actor, action, resource: Specific, result) :=
    cut(), actor.num = 2;

decide(actor, action, resource, result) :=
    _decide(actor, action, resource, result), cut(), result = "allow";

decide(actor, action, resource, result) := result = "deny";
