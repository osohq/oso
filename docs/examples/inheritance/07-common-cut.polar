## Policies where cut is common. ##

# Using cut occasionally is okay, but it isn't ideal for a policy that has many
# exceptions.  If we want, we can make exceptions an explicit part of our policy
# with the negation operator.

# This rule allows access if there is an `allowInner` rule that matches AND
# no `exception` rule that matches.
allow(user, action, resource) if
    allowInner(user, action, resource) and
    not exception(user, action, resource);

# Then, we can write `allowInner` rules to permit access (instead of the top
# level `allow`).
allowInner(user: User, "read", resource: PatientData) if
    user.role = "medical_staff" and
    user.treated(resource.patient) and
    not resource.private;

# And `exception` to restrict access.
exception(user: User, "read", _resource: Lab) if
    not user.medical_role = "lab_tech" or user.medical_role = "doctor";

# Not actually sure if this works... needs some testing. If it doesn't work
# we should think if there is some way to incorporate something like this, because
# cut is not ideal if it is common. (Although not sure if the idea of
# exception is much better). Instead we almost want an ALL operator which
# would change the matching for a given predicate call, ensuring that all
# match (instead of just one) before returning a true result.
#
# Prolog seems to have https://www.swi-prolog.org/pldoc/man?predicate=forall/2
# for this (maybe?).
