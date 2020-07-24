# Context

## NEW CONCEPTS:
# - defining an Env object to expose environment info.

allow(actor, _action, resource) if role(actor, "admin");

allow(_actor, _action, _resource) if new Env{}.var("ENV") = "development";
