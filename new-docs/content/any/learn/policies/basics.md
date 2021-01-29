---
title: Rule basics
weight: 1
---

# Rule basics

Policies are made up of [rules](polar-syntax#rules). Each rule defines a
statement that is either true or false. oso answers queries by evaluating rules
that match the query name and parameters. Let’s take a basic [allow
rule](glossary#allow-rules) as an example:

```polar
allow(actor, action, resource);
```

<!-- TODO(gj): link `Oso.is_allowed()` once API docs are hooked up. -->
When we use `Oso.is_allowed()` (or equivalent), we are making a query that asks
oso to evaluate all rules that match *(a)* on the rule name (`allow`), and
*(b)* on all the inputs.

In the rule above, `actor`, `action`, and `resource` are simply the parameter
names, i.e., they are variables that will match *anything*.

But if we replace `action` with a concrete type…

```polar
allow(actor, "read", resource);
```

…the rule will now only be evaluated if the second input exactly matches the
string `"read"`.

## `if`&nbsp;Statements

There are several ways to represent imperative `if` logic in Polar.

### In a Rule Body

The most common way to write an `if` statement in Polar is to add a body to a
rule. The following rule allows **any** actor to approve **any** expense
report:

```polar
allow(_actor, "approve", _report);
```

To restrict the rule such that only administrators may approve any expense
report, we can add a body:

```polar
allow(actor, "approve", _report) if
    actor.is_admin = true;
```

To express multiple truth conditions (e.g., `if A or B, then...`), we can
either create multiple rules…

```polar
allow(actor, "approve", _report) if
    actor.is_admin = true;

allow(actor, "approve", _report) if
    actor.title = "CFO";
```

…or we can use Polar’s [disjunction operator
(`or`)](polar-syntax#disjunction-or) to combine the conditions in a single rule
body:

```polar
allow(actor, "approve", _report) if
    actor.is_admin = true
    or actor.title = "CFO";
```

{{< callout "Tip" "green" >}}
  In these rules we declared some variables with leading underscores
  (`_actor`, `_report`). A leading underscore indicates that the variable will
  only be used once (Polar does not distinguish between definition and use).
  These variables are called *singleton variables*, and will match any value.
  To help prevent errors, a warning will be emitted if a singleton variable is
  not preceded by an underscore.
{{< /callout >}}

### As Specializers in a Rule Head

Given the following application class structure…

```polar
class User:
    ...

class Admin(User):
    ...
```

…we can modify our original bodiless rule to only allow `Admin` users to
approve any expense report by adding a
[specializer](application-types#registering-application-types) to the rule
head:

```polar
allow(_actor: Admin, "approve", _report);
```

The rule will fail when evaluated on a regular `User` and succeed when
evaluated on an `Admin`, encoding an implicit `if Admin` condition.

This is another example of the rule matching process: instead of matching
against a concrete value, we are instead checking to make sure the type of the
input matches the expected type — in this case, an `Admin`.

{{< callout "Tip" "green" >}}
  Try to use type specializers as often as possible. It will help make sure you
  don't accidentally allow access to an unrelated resource which happens to
  have matching fields.
{{< /callout >}}

## Combining Rules

Rules can be thought of as equivalent to methods in imperative programming. The
same idea should be applied when writing policies: any piece of logic that you
want to reuse throughout a policy can be extracted out into a new rule.

The benefits of that extraction are that it makes it easier to keep logic
consistent throughout and often results in a much more readable policy.

Take the following example. We want a rule saying that accountants can read
expenses. Our initial version might look like:

```polar
allow(user: User, "read", expense: Expense) if
    user.role = "accountant";
```

This would be fine, but if, for example, we wanted to allow the CFO to do
whatever an accountant can do, we would need to duplicate all the rules. Or if
we want to change how an application determines roles we would need to change
all locations using this.

So instead, we can refactor the role check into its own rule:

```polar
allow(user: User, "read", expense: Expense) if
    role(user, "accountant");

role(user, role_name) if
    user.role = role_name;
```

`role(user, "accountant")` is another example of pattern matching in Polar. Any
time a rule body contains a **predicate** like this, it is performing another
query. That is, it will try to find all *matching* rules named `role` with two
inputs.

You can also either use the [REPL](repl) or the `Oso.query_rule()` method to
interact with this directly. For example:

```python
from oso import Oso

class User:
    def __init__(self, name, role):
        self.name = name
        self.role = role

oso = Oso()
oso.load_str("role(user, role_name) if user.role = role_name;")

alice = User("alice", "accountant")
assert list(oso.query_rule("role", alice, "accountant"))
```

{{< callout "Tip" "green" >}}
  Try setting the `POLAR_LOG` environment variable before executing a polar
  query to see a [trace](tracing) of how the query is evaluated:

  ```console
  $ POLAR_LOG=1 python user.py
  [debug]   QUERY: role(<__main__.User object at 0x105da6190>, "accountant"), BINDINGS: {}
  [debug]     APPLICABLE_RULES:
  [debug]       role(user, role_name) if user.role = role_name;
  [debug]     RULE: role(user, role_name) if user.role = role_name;
  [debug]       QUERY: .(_user_5, "role", _value_1_7) and _value_1_7 = _role_name_6, BINDINGS: {_role_name_6 = "accountant", _user_5 = <__main__.User object at 0x105da6190>}
  [debug]         QUERY: .(_user_5, "role", _value_1_7), BINDINGS: {_user_5 = <__main__.User object at 0x105da6190>}
  [debug]           LOOKUP: <__main__.User object at 0x105da6190>.role()
  [debug]           => "accountant"
  [debug]         QUERY: _value_1_7 = _role_name_6, BINDINGS: {_role_name_6 = "accountant", _value_1_7 = "accountant"}
  [debug]   BACKTRACK
  [debug]           LOOKUP: <__main__.User object at 0x105da6190>.role()
  [debug]           => No more results.
  [debug]           BACKTRACK
  [debug]           HALT
  ```
{{< /callout >}}

## Summary

We covered some of the basics of policies, how to represent conditional logic,
and briefly touched on the core mechanic of pattern matching.
