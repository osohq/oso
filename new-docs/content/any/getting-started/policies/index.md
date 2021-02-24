---
title: Write Oso Policies (15 min)
weight: 3
any: false
aliases:
  - ../getting-started/policies/index.html
description: Learn about writing Oso policies - the source of truth for authorization logic.
---

# Write Oso Policies

Policies are the source of truth for the authorization logic used to evaluate
queries in Oso. As a reminder: Oso policies are written in a declarative
language called Polar. There is a full [Polar Syntax guide](polar-syntax) which
you can use as a reference of all the available syntax, but here we’ll give an
overview of getting started with writing policies.

The syntax might feel a bit foreign at first, but fear not: almost anything you
can express in imperative code can equally be expressed in Polar — often more
concisely and closer to how you might explain the logic in natural language.

{{% callout "Note" "green" %}}
Policies are stored in Polar files (extension `.polar`), which are loaded
into the authorization engine using the [Oso library](reference).
{{% /callout %}}

## Rule basics

Policies are made up of [rules](polar-syntax#rules). Each rule defines a
statement that is either true or false. Oso answers queries by evaluating rules
that match the query name and parameters. Let’s take a basic [allow
rule](glossary#allow-rules) as an example:

```polar
allow(actor, action, resource);
```

<!-- TODO(gj): link `Oso.is_allowed()` once API docs are hooked up. -->
When we use `Oso.is_allowed()` (or equivalent), we are making a query that asks
Oso to evaluate all rules that match *(a)* on the rule name (`allow`), and
*(b)* on all the inputs.

In the rule above, `actor`, `action`, and `resource` are simply the parameter
names, i.e., they are variables that will match *anything*.

But if we replace `action` with a concrete type…

```polar
allow(actor, "read", resource);
```

…the rule will now only be evaluated if the second input exactly matches the
string `"read"`.

### `if`&nbsp;Statements

There are several ways to represent imperative `if` logic in Polar.

#### In a Rule Body

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

{{% callout "Tip" "green" %}}
  In these rules we declared some variables with leading underscores
  (`_actor`, `_report`). A leading underscore indicates that the variable will
  only be used once (Polar does not distinguish between definition and use).
  These variables are called *singleton variables*, and will match any value.
  To help prevent errors, a warning will be emitted if a singleton variable is
  not preceded by an underscore.
{{% /callout %}}

#### As Specializers in a Rule Head

Given the following application class structure…

```polar
class User:
    ...

class Admin(User):
    ...
```

…we can modify our original bodiless rule to only allow `Admin` users to
approve any expense report by adding a
[specializer](#registering-application-types) to the rule
head:

```polar
allow(_actor: Admin, "approve", _report);
```

The rule will fail when evaluated on a regular `User` and succeed when
evaluated on an `Admin`, encoding an implicit `if Admin` condition.

This is another example of the rule matching process: instead of matching
against a concrete value, we are instead checking to make sure the type of the
input matches the expected type — in this case, an `Admin`.

{{% callout "Tip" "green" %}}
  Try to use type specializers as often as possible. It will help make sure you
  don't accidentally allow access to an unrelated resource which happens to
  have matching fields.
{{% /callout %}}

### Combining Rules

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

{{% callout "Tip" "green" %}}
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
{{% /callout %}}

### Summary

We covered some of the basics of policies, how to represent conditional logic,
and briefly touched on the core mechanic of pattern matching.

## Application Types

Any type defined in an application can be passed into Oso, and its attributes
may be accessed from within a policy. Using application types make it possible
to take advantage of an app’s existing domain model. For example:

{{< code file="policy.polar" >}}
allow(actor, action, resource) if actor.{{% exampleGet "isAdmin" %}};
{{< /code >}}

<!-- TODO(gj): Link `Oso.isAllowed()` once API docs are setup. -->

The above rule expects the `actor` variable to be a {{% exampleGet "langName"
%}} {{% exampleGet "instance" %}} with the field `{{% exampleGet "isAdmin" %}}`. The {{% exampleGet "langName" %}} {{% exampleGet "instance" %}} is passed
into Oso with a call to `Oso.{{% exampleGet "isAllowed" %}}()`:

{{% exampleGet "userClass" %}}

The code above provides a `User` object as the _actor_ for our `allow` rule.
Since `User` has a field called `{{% exampleGet "isAdmin" %}}`, it is checked
during evaluation of the Polar rule and found to be true.

In addition to accessing attributes, you can also call methods on application
instances in a policy:

{{< code file="policy.polar" >}}
allow(actor, action, resource) if actor.{{% exampleGet "isAdminOf" %}}(resource);
{{< /code >}}

### Registering Application Types

Instances of application types can be constructed from inside an Oso policy
using [the `new` operator](polar-syntax#new) if the class has been
**registered**. {{% exampleGet "registerClass" %}}

Once the class is registered, we can make a `User` object in Polar. This can be
helpful for writing inline test queries:

{{% exampleGet "testQueries" %}}

Registering classes also makes it possible to use
[specialization](polar-syntax#specialization) and [the `matches`
operator](polar-syntax#matches-operator) with the registered class.

In our previous example, the **allow** rule expected the actor to be a `User`,
but we couldn’t actually check that type assumption in the policy. If we
register the `User` class, we can write the following rule:

{{< code file="policy.polar" >}}
allow(actor: User, action, resource) if actor.name = "alice";
{{< /code >}}

This rule will only be evaluated when the actor is a `User`; the `actor`
argument is _specialized_ on that type. We could also use `matches` to express
the same logic on an unspecialized rule:

{{< code file="policy.polar" >}}
allow(actor, action, resource) if actor matches User{name: "alice"};
{{< /code >}}

Either way, using the rule could look like this:

{{% exampleGet "specializedExample" %}}

{{% callout "Note" "green" %}}
Type specializers automatically respect the **inheritance** hierarchy of
application classes. See the [Resources with
Inheritance](guides/more/inheritance) guide for an in-depth
example of how this works.
{{% /callout %}}

Once a class is registered, class or static methods can also be called from Oso
policies:

{{< code file="policy.polar" >}}
allow(actor: User, action, resource) if actor.name in User.superusers();
{{< /code >}}

{{% exampleGet "classMethodExample" %}}

### Built-in Types

Methods called on the Polar built-in types `String`, `Dictionary`, `Number`,
and `List` punt to methods on the corresponding application language class.
That way you can use familiar methods like `.{{% exampleGet "startswith" %}}()`
on strings regardless of whether they originated in your application or as a
literal in your policy. This applies to all of Polar's [supported
types](polar-syntax#primitive-types) in any supported application language. For
examples using built-in types, see [the {{% exampleGet "langName" %}}
library](reference/polar/classes) guide.

{{% callout "Warning" "orange" %}}
Do not attempt to mutate a literal using a method on it. Literals in Polar
are constant, and any changes made to such objects by calling a method will
not be persisted.
{{% /callout %}}


### Summary

- **Application types** and their associated application data are available
  within policies.

* Types can be **registered** with Oso, in order to:
  - Create instances of application types in policies
  - Leverage the inheritance structure of application types with **specialized
    rules**, supporting more sophisticated access control models.

- You can use **built-in methods** on primitive types & literals like strings
  and dictionaries, exactly as if they were application types.
