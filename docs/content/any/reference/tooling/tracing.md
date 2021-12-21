---
title: Tracing
aliases:
    - ../more/dev-tools/tracing.html
description: Polar tracing shows logs of how a query is evaluated.
---

# Tracing

Polar's tracing support prints logs of how a query is evaluated. It is available in both `INFO` and `TRACE` level outputs.

## Enabling `INFO` output 

`INFO` traces are viewed by setting the environment variable: `POLAR_LOG=info`. The `INFO` output is a more concise subset of `TRACE` and is intended to by used by those developing and debugging polar policies in their local environment. 

It will print out `[oso][info]` messages for many notable points of the query execution, for example:

 * Queries and sub-query evaluations.
 
 * Source information for the rules which were applicable to each query, and a warning if none were found to be applicable. 
 
 * Query successes and the contents of any variable bindings returned by Oso.
 
## Enabling `TRACE` output

`TRACE` logs are viewed by setting an environment variable: `POLAR_LOG=trace`.

`POLAR_LOG=trace` exposes a more verbose view of a query execution in Polar and extends the contents of `POLAR_LOG=info`. The `POLAR_LOG=trace` traces will emit a log for every Goal encountered by the Polar virtual machine during the query execution. Notable examples include:

* Showing parameter type checking information during query applicability tests.

* Showing values that are bound to variables for each expression.

* Showing values returned from calls into the application.


### Example

{{<literalInclude path="reference/tooling/info" tabGroup="tracing">}}
{{<literalInclude path="reference/tooling/trace" tabGroup="tracing">}}

All `POLAR_LOG` output can be disabled by setting `POLAR_LOG=0` or `POLAR_LOG=off`.
