# Context

## NEW CONCEPTS:
# - defining an Env object to expose environment info.

allow(actor, action, resource) := role(actor, "admin");

allow(actor, action, resource) := Env{}.var("ENV") = "development";
