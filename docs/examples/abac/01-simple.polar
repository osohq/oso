# Simple ABAC

## NEW CONCEPTS:
# - allow rule with a simple string comparison attribute check

# rule-start
allow(actor: User, "view", resource: Expense) :=
    resource.submitted_by = actor.name;
# rule-end
