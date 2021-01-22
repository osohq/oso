---
title: Basics
weight: 1
---

## Rule Basics

Policies are made up of rules. Each rule defines
a statement that is either true or false. oso answers queries by evaluating rules that match the
query name and parameters. Let’s take a basic allow rule as an example:

```
allow(actor, action, resource) if ...
```

When we use `is_allowed()` (or equivalent), we are making a query that asks oso to
evaluate all rules that match *(a)* on the rule name `"allow"`, and *(b)* on all the inputs.

In the rule above, `actor`, `action`, and `resource` are simply the parameter names,
i.e. they are variables that will match *anything*.

But if we replace `action` with a concrete type:

```
allow(actor, "read", resource);
```

the rule will now only be evaluated if the second input exactly matches the string `"read"`.

## `if` Statements

There are several ways to represent imperative `if` logic in Polar.

### In a Rule Body

The most common way to write an `if` statement in Polar is to add a body to
a rule. The following rule allows **any** actor to approve **any** expense report:

```
allow(_actor, "approve", _report);
```

To restrict the rule such that only administrators may approve any expense
report, we can add a body:

```
allow(actor, "approve", _report) if
    actor.is_admin = true;
```

To express multiple truth conditions (e.g., `if A or B, then...`), we can
either create multiple rules…

```
allow(actor, "approve", _report) if
    actor.is_admin = true;

allow(actor, "approve", _report) if
    actor.title = "CFO";
```

…or we can use Polar’s Disjunction (or) operator (OR) to combine the conditions
in a single rule body:

```
allow(actor, "approve", _report) if
    actor.is_admin = true
    or actor.title = "CFO";
```

### As Specializers in a Rule Head

Given the following application class structure…

```
class User:
    ...

class Admin(User):
    ...
```

…we can modify our original bodiless rule to only allow `Admin` users to
approve any expense report by adding a specializer to the
rule head:

```
allow(_actor: Admin, "approve", _report);
```

The rule will fail when evaluated on a regular `User` and succeed when
evaluated on an `Admin`, encoding an implicit `if Admin` condition.

This is another example of the rule matching process: instead of matching against
a concrete value, we are instead checking to make sure the type of the input
matches the expected type - in this case, an `Admin`.

## Combining Rules

Rules can be thought of as equivalent to methods in imperative programming.
And the same idea should be applied when writing policies: any pieces
of logic that you want to reuse throughout a policy can be extracted out into
a new rule.

The benefit of this is (a) it makes it easier to keep logic consistent throughout,
and (b) it often results in much more readable policy.

Take the following example. We want a rule saying that accountants
can read expenses.
Our initial version might look like:

```
allow(user: User, "read", expense: Expense) if
  user.role = "accountant";
```

This would be fine, but if, for example, we wanted to allow the CFO to
do whatever an accountant can do, we would need to duplicate all the rules.
Or if we want to change how an application determines roles we would need
to change all locations using this.

So instead, we can refactor the role check into its own rule:

```
allow(user: User, "read", expense: Expense) if
  role(user, "accountant");

role(user, role_name) if user.role = role_name;
```

The `role(user, "accountant")` is yet another example of matching happening
in Polar. Any time a rule body contains a **predicate** like this, it is performing
another query. I.e. it will try and find all *matching* rules called “role” with
two inputs.

You can also either use the The REPL or the `oso.query_rule`
method to interact with this directly. For example:

```
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

## Summary

We covered some of the basics of policies, how to represent conditional
logic, and the core idea of matching.
