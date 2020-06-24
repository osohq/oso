# Deny all mallories
_deny(actor, action, resource, reason) := actor.name = "Mallory",
    reason = "Actor in blacklist";

_allow(actor, action, resource: {name: "allowed"});
_allow(actor, action, resource: {name: "allowed2"});

# Deny rules take precedence due to cut().
decide(actor, action, resource, "deny", reason) :=
    _deny(actor, action, resource, reason), cut();

# If there was no deny, this branch will be evaluated.
decide(actor, action, resource, "allow", _) :=
    _allow(actor, action, resource), cut();

# If neither matched, then deny
decide(actor, action, resource, "deny", "Default deny");
