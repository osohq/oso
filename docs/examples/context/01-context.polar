# Context

## NEW CONCEPTS:
# - defining an Env object to expose environment info.

# admin-start
allow(actor, _action, _resource) if role(actor, "admin");
# admin-end

# env-start
allow(_actor, _action, _resource) if Env.var("ENV") = "development";
# env-end
