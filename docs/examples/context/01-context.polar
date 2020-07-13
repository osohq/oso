# Context

## NEW CONCEPTS:
# - defining an Env object to expose environment info.

allow(actor, action, resource) if role(actor, "admin");

allow(actor, action, resource) if new Env{}.var("ENV") = "development";
