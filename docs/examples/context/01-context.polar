# Context

## NEW CONCEPTS:
# - defining an Env object to expose environment info.

# admin-start
allow(actor, _action, _resource) if actor = "admin@example.org";
# admin-end

# env-start
allow(_actor, _action, _resource) if Env.var("ENV") = "development";
# env-end
