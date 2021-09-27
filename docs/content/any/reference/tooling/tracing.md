---
title: Tracing
aliases:
    - ../more/dev-tools/tracing.html
description: Polar tracing shows logs of how a query is evaluated.
---

# Tracing

Polar tracing shows logs of how a query is evaluated.

## Enabling Tracing

Debug mode is enabled by setting an environment variable: `POLAR_LOG=1`.

It will print out `[debug]` messages during query evaluation that show how the query is executed.
Notable things include:


* Queries and sub-query evaluations.


* Showing values that are bound to variables for each expression.


* Showing which rules a Polar predicate call will evaluate.


* Showing values returned from calls into the application.

### Example

```
query> f(12);
[debug]   QUERY: f(12), BINDINGS: {}
[debug]     APPLICABLE_RULES: [
[debug]       f(x) if new (Foo{x: x}, _instance_2) and .(_instance_2, foo(), _value_1) and _value_1 = x;
[debug]       f(x) if x = 1;
[debug]       f(x) if x + 1 and _op_3 == 2;
[debug]     ]
[debug]     RULE:
[debug]     f(x) if
[debug]       new Foo(x: x).foo() = x
[debug]       QUERY: new (Foo{x: _x_10}, _instance_2_11) and .(_instance_2_11, foo(), _value_1_12) and _value_1_12 = _x_10, BINDINGS: {"_x_10": "12"}
[debug]         QUERY: new (Foo{x: _x_10}, _instance_2_11), BINDINGS: {"_x_10": "12"}
[debug]         QUERY: .(_instance_2_11, foo(), _value_1_12) and _value_1_12 = _x_10, BINDINGS: {"_instance_2_11": "Foo{x: 12}", "_x_10": "12"}
[debug]           QUERY: .(_instance_2_11, foo(), _value_1_12), BINDINGS: {"_instance_2_11": "Foo{x: 12}"}
[debug]             LOOKUP: Foo{x: 12}.foo()
[debug]             => 13
[debug]           QUERY: _value_1_12 = _x_10, BINDINGS: {"_x_10": "12", "_value_1_12": "13"}
[debug]             BACKTRACK
[debug]             LOOKUP: Foo{x: 12}.foo()
[debug]             => No more results.
[debug]             BACKTRACK
[debug]     RULE:
[debug]     f(x) if
[debug]       x = 1
[debug]       QUERY: _x_13 = 1, BINDINGS: {"_x_13": "12"}
[debug]         BACKTRACK
[debug]     RULE:
[debug]     f(x) if
[debug]       x + 1 == 2
[debug]       QUERY: _x_14 + 1 and _op_3_15 == 2, BINDINGS: {"_x_14": "12"}
[debug]         QUERY: _x_14 + 1, BINDINGS: {"_x_14": "12"}
[debug]           MATH: 12 + 1 = _op_3_15, BINDINGS: {}
[debug]         QUERY: _op_3_15 == 2, BINDINGS: {"_op_3_15": "13"}
[debug]           CMP: 13 == 2, BINDINGS: {}
[debug]           BACKTRACK
[debug]           HALT
False
```

Debug mode can be disabled by setting `POLAR_LOG=0` or `POLAR_LOG=off`.