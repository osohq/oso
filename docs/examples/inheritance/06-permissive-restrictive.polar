# NOTE: This is not included in the example right now but would be a good future
# thing to add.

# More permissive access with inheritance

# Suppose that Order data needs to be read by billing department members to make
# sure the hospital is charging correctly for each service provided.  This would
# be a more permissive policy (a rule that adds access on top of others).

allow(user: User, "read", _resource: Order) if
    user.role = "billing";

# Now, billing dept members can read orders! This rule combines with the other
# rules we have already written.

# More restrictive access is also possible.  What if we only want
# lab technicians and doctors to be able to read the Lab resource.

allow(user: User, "read", resource: Lab) if
    cut and
    (user.medical_role = "lab_tech" or user.medical_role = "doctor") and
    user.treated(resource.patient);

# This rule relies on two features of Polar:
# 1. Rule order is defined based on specializers. A rule over a subclass (Lab)
#    will execute before a rule defined for a superclass (PatientData). More details in [LINK].
# 2. The cut operator stops the Polar engine from running additional rules.
#    We need this since Polar by default will execute all rules that match a
#    given operation.
