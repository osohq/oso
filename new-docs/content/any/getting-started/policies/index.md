---
title: Write Oso Policies (15 min)
weight: 3
any: false
aliases:
  - ../getting-started/policies/index.html
description: Learn about writing Oso policies - the source of truth for authorization logic.
---

# Write Oso Policies

This tutorial will ...

# What is a policy in Oso?

An _authorization policy_ is a set of logical _rules_ for who
is allowed to access what resources in an application.
Some examples of rules expressed in English are:
* The user named "Banned Bad User" may not access any resources.
* No user may access any resource before 06:00 in their local timezone.
* A user may access any resource that they created.

In any particular application, such rules may be represented
and enforced in many different ways; e.g., as `if` statements
in code, with database access control, filesystem permissions, etc.

Oso is a library that makes it easy to express and enforce
authorization policies. Policies are kept separate from both
application code and the underlying database/filesystem/etc.
But because Oso is a library that runs in your application
process, it has direct access to application objects and types,
which lets you express application-specific policies in a very
natural way.

Authorization policies in Oso are expressed in a declarative,
logic-based programming language called Polar. You've probably
already seen some Polar policies in the [Quickstart](quickstart)
and [Add to an App]({{< ref path="/getting-started/application/index.md" lang="python" >}})
guides, and the [Polar Syntax guide](polar-syntax) has a detailed
description of the language syntax and features.

The purpose of _this_ guide is to serve as a tutorial introduction
to the Polar language. When you've finished it, you should be able
to read and write simple Polar rules over your own application objects
and types.

If you have some prior exposure to logic programming, the Polar
language will feel very familiar. If not, don't panic! Polar is
designed to be simple, often allowing easier and more concise
expression of authorization rules than equivalent code in an
imperative language. If you can express an authorization rule
as a simple declarative sentence of English (or other natural language),
it should be straightforward to turn into a Polar rule.

If you're unsure of what _kind_ of authorization policy to write,
you may want to head over to the [Conceptual Guides](learn) or
start with a familiar pattern such as [roles](/guides/roles).
This tutorial will not be concerned with a specific kind of policy,
but rather with the language in which many different kinds of
policies may be expressed.

## Application Setup

In this guide, we'll use the sample expenses app from the
[Quickstart](quickstart) guide as an example. It provides a
simple `Expense` class that models an expense record (e.g.,
for a work-related purchase) that a user might create, read,
update, or delete.

An authorization policy for such an application might include
rules such as:
* A user may read any `Expense` that they submitted.
* A user may approve an `expense` instance if they manage the
  user that submitted it and the amount is less than some maximum.

The rest of this tutorial will be concerned with how to turn
informal English rules like the above into concrete Polar rules
that the Oso library can enforce.

## Queries

_FIXME: This suction sucks, but something like it is needed._

Before we can fully explore policies, however, we'll talk
briefly about how a policy is _used_ by Oso to make authorization
decisions.

To enforce policy decisions in your application, you call
the Oso library function `is_allowed(actor, action, resource)`,
supplying arbitrary objects for the three arguments. That
function _queries_ the policy engine for the Polar term
`allow(actor, action, resource)`, passing (references to)
the arguments along to the query engine. If the query is
successful (i.e., returns more than zero matching results),
`is_allowed` returns true; otherwise, it returns false.

An analogy to a relational database may be helpful here.
A policy is like a set of tables, populated with data needed
to make allow/deny authorization decisions. A query over those
tables may include arguments that a row must match in some way
to be returned as part of the result set.

## Trivial Policies

Policies in Oso are made up of [rules](polar-syntax#rules).
A Polar policy file (extension `.polar`) consists of zero
or more rules, each terminated by a semicolon (`;`).
It may also contain comments (which start with `#`) and
[inline queries](polar-syntax#inline-queries) (which begin
with `?=`) for testing.

Let's take the base case first: a policy with zero rules.
If you don't load any Polar files (or load only empty ones),
you get the _empty policy_, which has no rules defined. This
is a perfectly valid policy, albeit not a very interesting one.
Every query against the empty policy fails, because there are
no rules that match. This means that nothing is authorized,
i.e., _everything is denied by default_.

Suppose we wanted the opposite default — to _allow_ everything.
We can achieve that by loading a policy that defines a single
rule that matches any three arguments:

```polar
allow(actor, action, resource);
```

Since this is our first rule definition, let's step through it
in detail. The name of the rule is `allow`. It has three parameters,
enclosed in parenthesis after the name: here they are _variables_
named `actor`, `action`, and `resource`. The name and parameters
together are called the _head_ of a rule. We'll see _bodies_ shortly,
but this rule doesn't have one, since the terminating `;` comes
immediately after the head.

Now let's talk about how rule definitions are used during a query.
When a rule definition is loaded from a policy, it doesn't _do_
anything; like a function definition, it is code that may be evaluated,
but only in response to a "call" or query.

Many queries in Polar look superficially like function calls:
`allow(1, 2, 3)` is a possible query. (You can use the Polar REPL to
interactively query Polar; this can be useful for learning the language
and for testing policies.) But queries are not exactly like function
calls; in particular, they can only "return" true or false — they denote
[logical predicates](https://en.wikipedia.org/wiki/Predicate_(mathematical_logic)).
And in fact it's best not to think of them as returning anything;
rather, we say a query either _succeeds_ (the predicate is true) or
_fails_ (it is false).

Queries for certain expressions may be answered even without any rule
definitions. For example, the following queries all succeed:
* `1 = 1`
* `1 < 2`
* `x = 1 and x < 2`

But a query for a predicate (e.g., an `allow` query for authorization)
can only succeed if:
* The name matches a defined rule, _and_
* That rule has the same number of parameters as arguments
  supplied to the query, _and_
* Each of the supplied arguments match the corresponding parameters
  from the rule head, _and_
* Queries for each term in the rule's body (if any) all succeed.

Arguments match parameters by [unification](polar-syntax#unification),
which is a binding/equality-checking operator: an unbound variable
unifies with a value by binding to it, a bound variable unifies with
a value if its value does, and two (non-variable) values unify if
they are equal.

In the case of the rule definition above, each of the three variable
parameters will unify with _any_ supplied argument. Since there is
no body to restrict these variables' values, this rule will match any
query of the form `allow(x, y, z)`. Hence, everything is allowed.

Now, if you try to actually load the above rule, you'll get a warning
about [singleton variables](polar-syntax#singletons). That's Polar
telling you that the variables aren't used, and so the rule might
be buggy, or the variable names misspelled. In this case it's not
an error, so we can silence the warning by prefixing the variable
names with underscores:

```polar
allow(_actor, _action, _resource);
```

That means exactly the same thing, it just tells Polar not to
tell us that the variables are singletons. We can even take it
one step further, and use the special _anonymous variable_ `_`:

```polar
allow(_, _, _);
```

Each occurrence of the anonymous variable `_` is considered
a unique variable, so this rule also accepts any three
possibly-but-not-necessarily distinct arguments. It is the
minimal "allow everything" rule.

## Non-trivial Policies

The trivial policies (deny/allow everything) are obviously not useful
in real-world applications. The goal of a real policy is to restrict
the allowed values of `actor`, `action`, and `resource` _just enough_
to allow authorized requests and deny everything else.

One simple way to do this is to replace variable parameters with
literals that must be matched, for example:

```polar
allow(_actor, "GET", _resource);
```

This rule allows arbitrary first and third arguments (singleton
variables `_actor` and `_resource`), but the second argument
(the action) must match the literal string `"GET"`. If the supplied
action were, say, `"POST"`, that argument, and therefore the rule,
would fail to match, so the query would fail, and so authorization
would be denied.

Suppose we started with the above, but soon realized that we also
needed to allow `"POST"` and `"DELETE"` requests. There are several
ways to write this, but the simplest is to just add more rules that
match those actions:

```polar
allow(_actor, "GET", _resource);
allow(_actor, "POST", _resource);
allow(_actor, "DELETE", _resource);
```

Here we have three `allow` rules, each of which matches a different
value of its second argument. A query may succeed by matching any
of them; e.g., the query `allow("foo", "GET", "bar")` will succeed
by matching (only) the first rule, while `allow("foo", "DELETE", "bar")`
would match (only) the third. The query `allow("foo", "FROB", "bar")`
would fail, because it does not match any of them.

## Conditional Rules

Another way to express the policy above would be to use a single
rule that checked for each of the three actions we wish to allow.
We can write that by using a _conditional_ rule, i.e., one that
has the keyword `if` after its parameter list, followed by zero
or more _body_ terms that serve to restrict its applicability:

```polar
allow(_actor, action, _resource) if
    action = "GET" or
    action = "POST" or
    action = "DELETE";
```

Here we've recoded the implicit disjunction expressed by three rules
as an explicit disjunction of conditions using the logical `or` operator.
(As you might guess, there are also logical `and` and `not` operators.
Like `or`, they may only be used in rule bodies, i.e., after an `if`.)

Yet another way to write this policy would be to use the `in` operator:
```polar
allow(_actor, action, _resource) if
    action in ["GET", "POST", "DELETE"];
```

Here the list of allowed actions is encoded as a literal Polar list
(comma separated terms between square brackets). The `in` operator
is a membership check: it succeeds when its left-hand side (`action`)
appears anywhere in its right-hand side (the list `["GET", "POST",
"DELETE"]`), which is true just when the left-hand side equals the
list's first element, *or* its second element, *or* its third
element, .... So this rule means the same thing as the ones above.

## Objects, Types, and Fields

In our example app, an action is represented by a string, but the
resources we're protecting are instances of our `Expense` model class.
Let's use its `submitted_by` field to implement the sample rule:

* A user may read any `Expense` that they submitted.

One way to write such a rule in Polar is like this:

```polar
allow(actor, "GET", resource) if
    resource.submitted_by = actor;
```

The Polar term `resource.submitted_by` refers to the `submitted_by`
field of the `resource` object. Comparing that value to the `actor`
represents the "that they submitted" constraint of the English rule
above.

This rule, however, has a problem: it assumes that there _is_
a `submitted_by` field on the `resource` object. If we happened
to pass a different kind of object, this rule would fail, but in
a potentially confusing and error-prone way. To fix this, we can
add an explicit type check using the `matches` operator:

```polar
allow(actor, "GET", resource) if
    resource matches Expense and
    resource.submitted_by = actor;
```

Here `Expense` is the application model class, and `matches` checks
that its left-hand side is an instance of its right (or any subclass
thereof). If that type check fails, the next term won't be evaluated.

In order for Polar to use a class in a type check, it must be
_registered_. If you're using an ORM adapter, this happens
automatically for all model classes; otherwise, you'll have
to register classes manually using `Oso.register_class`. If
you forget, Polar will raise an error at query time.

### Specializers

Since type checks are common and helpful for rule arguments,
Polar provides a shortcut syntax:

```polar
allow(actor, "GET", resource: Expense) if
    resource.submitted_by = actor;
```

This rule only matches a query whose third argument is an instance
of the `Expense` class, and so we may safely access fields we know
exist in such instances. These type restrictions are called
_specializers_, and we say that, e.g., the `resource` argument
is _specialized_ on the `Expense` class.

You may specialize on any argument; if our actors were instances
of a `User` class, for instance, we could write:

```polar
allow(actor: User, "GET", resource: Expense) if ...;
```

### Built-in Classes

We said above that any class you wish to use as a specializer
(or on the right-hand side of the `matches` operator) must be
registered with Polar. For convenience, Polar automatically
registers a few classes, such as:

* `Dictionary`
* `String`
* `List`
* `Integer`
* `Float`

These classes correspond to ones in the application language,
e.g., in Python, `String` is actually the built-in `str` class.

## Method Calls

In addition to accessing fields in a rule, you can also call methods
on application objects. We'll demonstrate this using the built-in
`String` class, but it applies to instances any registered class.

Suppose we decided to change our encoding of actions so that
every "read" action started with the letter "r". We could allow
all such actions using a rule like this:

```polar
allow(_actor, action: String, _resource) if
    action.startswith("r");
```

## Other Rules

So far all of our examples have involved only `allow` rules
with three parameters. That's a natural place to start with Oso,
because the `is_allowed` function generates queries of the form
`allow(actor, action, resource)`. But you can write rules for
whatever you like, and query them from within your `allow` rules.
For instance, the built-in [roles library](/guides/roles) supplies an
`allow` rule that looks like this:

```polar
allow(user, action, resource) if
    resource_role_applies_to(resource, role_resource) and
    user_in_role(user, role, role_resource) and
    role_allow(role, action, resource);
```

In order for this `allow` rule to succeed, each of the body terms
must succeed, and so the rules `resource_role_applies_to`, `user_in_role`,
and `role_allow` must be defined and match the supplied arguments.
Rules may also be recursive, i.e., may refer to themselves.
