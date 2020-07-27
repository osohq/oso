## Policies where cut is common. ##

# Using cut occasionally is okay, but it isn't ideal for a policy that has many
# exceptions.  If we want, we can make exceptions an explicit part of our policy
# with the negation operator.

# This rule allows access if there is an `allowInner` rule that matches AND
# no `exception` rule that matches.
allow(actor, action, resource) if
    allowInner(actor, action, resource) and
    not exception(actor, action, resource);

# Then, we can write `allowInner` rules to permit access (instead of the top
# level `allow`).
allowInner(actor: Actor, "read", resource: PatientData) if
    actor.role = "medical_staff" and
    actor.treated(resource.patient) and
    not resource.private;

# And `exception` to restrict access.
exception(actor: Actor, "read", resource: Lab) if
    not actor.medical_role = "lab_tech" or actor.medical_role = "doctor";

# Not actually sure if this works... needs some testing. If it doesn't work
# we should think if there is some way to incorporate something like this, because
# cut is not ideal if it is common. (Although not sure if the idea of
# exception is much better). Instead we almost want an ALL operator which
# would change the matching for a given predicate call, ensuring that all
# match (instead of just one) before returning a true result.
#
# Prolog seems to have https://www.swi-prolog.org/pldoc/man?predicate=forall/2
# for this (maybe?).
