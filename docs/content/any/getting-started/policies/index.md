---
title: Write Oso Policies (30 min)
weight: 3
any: false
aliases:
  - ../getting-started/policies/index.html
description: Learn about writing Oso policies - the source of truth for authorization logic.
---

# Write Oso Policies

This tutorial will teach you how to write policies for the
Oso authorization system. You will learn what policies are,
how they're queried, and the basic structure and operations
of the rules that comprise them. You will also learn how to
write rules that refer to application classes, instances,
and fields.

# What is a policy in Oso?

An **authorization policy** is a set of logical **rules** for
who is allowed to access what resources in an application.
Some examples of rules expressed in English are:

* The user named "Banned B. User" may not access any resources.
* No user may access any resource before 06:00 in their local timezone.
* A user may access any resource that they created.

In any particular application, such rules may be represented
and enforced in many different ways; e.g., as `if` statements
in code, with database access control, filesystem permissions, etc.

Oso is a library designed to express and enforce
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
designed to be simple, often allowing more concise
expression of authorization rules than equivalent code in an
imperative language. If you can express an authorization rule
as a simple declarative sentence of English (or other natural language),
it should be straightforward to turn into a Polar rule.

If you're unsure of what _kind_ of authorization policy to write,
you may want to head over to the [Conceptual Guides](learn) or
start with a familiar pattern such as [roles](/learn/roles).
This tutorial will not be concerned with a specific kind of policy,
but rather with the language in which many different kinds of
policies may be expressed.

## Application Setup

To use Oso for authorization, your application must first:

1. Load the Oso library: `from oso import Oso`
2. Create an `Oso` instance: `oso = Oso()`
3. Load a policy: `oso.load_file(policy)`.

Then, at authorization time, you can call:

```python
oso.is_allowed(actor, action, resource)
```

This returns a boolean indicating whether the given `actor`
is authorized to perform `action` on `resource` according to
the current policy.

The arguments to `oso.is_allowed` may be arbitrary Python
objects. We'll often use strings for examples, but in your
application they could be ORM model objects, URIs, numbers, etc.

## Rules

The way you control whether `oso.is_allowed` returns `True`
or `False` is to define and load Polar rules that **match**
only the desired set of `actor`, `action`, and `resource`
arguments. This matching may involve logical connectives,
type checks, equality checks, variable bindings, field lookups,
method calls, arithmetic, comparisons, etc. See the
[Polar Syntax guide](polar-syntax) for a complete list
of available operators.

Let's start with the basic syntax. A **rule definition** in Polar
has a **name**, a **parameter list**, and an optional **body**.
Here's a very simple rule definition:

```polar
allow("Zora", "read", "document-1");
```

The name of this rule is `allow`. It has three string-valued
parameters: `"Zora"`, `"read"`, and `"document-1"`, which must
match the supplied arguments exactly. The terminating semicolon
comes right after the parameter list, so it has no body; we say
that this rule is **unconditional**.

Here's a **conditional** rule definition:

```polar
allow("Zora", "read", "document-1") if 1 = 0;
```

Like the previous rule, this one tries to match the query arguments
exactly. But this rule also has a body, introduced by the keyword
`if` after the parameter list. The body has one condition, which
must also be true in order for this rule to match a query. (_We_
can see that that the condition `1 = 0` is always false, but Polar
does not know that; it must perform the comparison each time.)
We could add more conditions with the logical connective `and`:

```polar
allow("Zora", "read", "document-1") if
    1 = 0 and
    0 = 1;
```

The logical operators `or` (binary) and `not` (unary) can also
be used in a rule body, as well as a variety of mathematical,
matching, and lookup operators.

## The Big Picture

Before diving any deeper into the details of rules and matching,
let's take a moment to put them in context. Rules are loaded into
a **knowledge base**, a kind of specialized in-memory database that
supports **queries** by pattern matching and logical inference.

![Oso Architecture](getting-started/policies/arch.svg)

To determine whether a given argument tuple is authorized,
the Oso library issues a query to the knowledge base.
Abstractly, a query means: "Given the rules you know, is
this expression true?" If the expression can be proved true,
the query **succeeds** with some results (variable bindings)
indicating how; otherwise, it **fails**.

If the knowledge base is empty, nothing is known to be true,
so _every_ query will fail, and so _nothing_ is authorized.
This makes Oso _deny by default_.

If the knowledge base is non-empty, then a given query is true
if it successfully matches one or more known rules. A query
matches a rule if:

* The names match, _and_
* Each query argument matches the corresponding rule parameter, _and_
* Queries for each condition in the rule body all succeed.
  (Note that this is trivially true if the rule has no body.)

Let's make this concrete by considering a very simple policy:
suppose users named Abagail, Carol, and Johann are allowed to
read some document. We'll represent users, actions, and documents
as strings for now, but we'll see richer representations in just
a moment. We could express this policy with the following three
Polar rule definitions:

```polar
allow("Abagail", "read", "document-1");
allow("Carol", "read", "document-1");
allow("Johann", "read", "document-1");
```

After we load this file into the knowledge base, we can make
authorization decisions from our Python application by
calling:

```python
oso.is_allowed("Johann", "read", "document-1")
```

The Oso library issues a Polar query to the knowledge base:

```polar
allow("Johann", "read", "document-1")
```

The knowledge base then searches for a definition that matches the
query. For the query to succeed, it must find at least one match.
In this case, the first rule definition fails to match, because its
first parameter does not match the first argument: `"Abagail" != "Johann"`.
The second rule also fails to match, because `"Carol" != "Johann"`.
The third rule, however, successfully matches each of the arguments
with the corresponding parameter: `"Johann" = "Johann"`, `"read" = "read"`,
and `"document-1" = "document-1"`. So the query succeeds,
and `oso.is_allowed` returns `True`.

## Variables

In the example above, `allow` rules were used to explicitly
enumerate sets of permissions using exact matching (value equality).
That style of rule is sometimes useful, but such policies can become
unmanageably large. The Polar knowledge base does index rules,
so policies containing tens or hundreds of thousands of rules like
that can still be used efficiently should your application and
policy require it, but it's not really recommended.

What we usually want instead is to write rules that match more
than one argument by exploiting regularities or abstractions in
our application objects and policy. For instance, each of the three
rules above has the same second and third parameters, so we can
collapse all three rules into one if we _conditionally_ match the
first argument.

We can do that by using a **variable** parameter named `actor`
(instead of a literal string), and checking whether its value
matches any of the three allowed names:

```polar
allow(actor, "read", "document-1") if
    actor = "Abagail" or
    actor = "Carol" or
    actor = "Johann";
```

We would read this in English as: "allow an actor to read document-1
if the actor is Abagail, or it is Carol, or is Johann".

### Bindings

Let's look now in detail at what happens when a query is matched
against a rule like the one above. Suppose the query is:

```polar
allow("Zora", "read", "document-1")
```

Polar first matches the names (`allow`), then tries to match
each of the arguments `("Zora", "read", "document-1")` with
the parameters `(actor, "read", "document-1")`. These all succeed,
but in two different ways: the string `"Zora"` matches the variable
`actor` by **binding** the variable to the string, while the latter
two arguments match the corresponding parameters by value (string)
equality.

This operation of either binding an unbound variable _or_
comparing two values (of bound variables) is called
[**unification**](https://en.wikipedia.org/wiki/Unification_(computer_science)).
It happens implicitly when Polar matches query arguments
with rule parameters, and you can also use it explicitly
(e.g., in a rule body) with the [`=`](polar-syntax#unification)
operator, which we read as "equals".

Sometimes it's useful to distinguish the binding aspect of
unification from equality checking, so Polar also offers an
[assignment operator `:=`](polar-syntax#assignment) which
raises an error if its left-hand side isn't an unbound variable
(unification can bind either side), and an [equality operator
`==`](polar-syntax#numerical-comparison) which will never
bind a variable.

Once a variable is bound, there is no way to reassign or change
its value. This is because references to a bound variable are
replaced by the variable's value. For instance, `x = 1 and x = 2`
fails, because once `x` is bound to `1`, it can't be rebound,
and its value isn't equal to `2`.

Going back to our example above, once the variable `actor` is
bound to the supplied argument `"Zora"`, subsequent uses of it
are automatically dereferenced, i.e., replaced with `"Zora"`.
So when Polar queries for the first body condition,
`actor = "Abagail"` becomes `"Zora" = "Abagail"`, which fails.

You should try to work through for yourself what happens with
the query:

```polar
allow("Johann", "read", "document-1")
```

## Instances and Fields

We still have an explicit enumeration of permissions in the rule
above. That's fine for users represented by strings and such, but
most applications use more structured representations for their
actors, actions, and resources.

Suppose then that our actors are represented by instances of a
Python `User` class, with, say, a user ID and administrator
flag:

```python
@dataclass
class User:
    id: int
    admin: bool
```

Let's also assume also a simple `Document` class, also with an
ID and an `owner` field that references a user ID (in a real app
these would both be ORM model classes):

```python
@dataclass
class Document:
    id: int
    owner: int
```

The two policy rules we'll implement are:

* A user that is an administrator may read any document.
* A user may read any document that they own.

Our application will be making calls like this:

```python
oso.is_allowed(User(id=0, admin=True), "read", Document(id=1, owner=0))
```

The Oso library will generate a Polar query like this:

```polar
allow(User{id: 0, admin: true}, "read", Document{id: 1, owner: 0})
```

One way to define policy rules that match such queries is to bind
`user` and `document` variables to the first and third arguments,
then use the `.` ("dot") operator to lookup field values in the
bodies:

```polar
# A user that is an administrator may read any document.
allow(user, "read", document) if
    user.admin = true;

# A user may read any document that they own.
allow(user, "read", document) if
    user.id = document.owner;
```

Lookups happen at query time, on the live application objects.
Polar uses a [foreign-function
interface](https://en.wikipedia.org/wiki/Foreign_function_interface)
to perform the lookup and pass the result back to the query engine.

For the query above, both of these rules would match, since the
lookups would yield values that satisfy the conditions. If the call
were, however, something like:

```python
oso.is_allowed(User(id=1, admin=False), "read", Document(id=1, owner=0))
```

This call would return `False`, since neither rule would match
the supplied instances. The first rule would fail because the
`admin` flag isn't true, and the second because the user's ID
`1` isn't equal to the document's owner ID `0`.

## Classes

There's a potential problem with the rules above: they refer to fields
of objects without checking the **types** of those objects. For example,
if we were to pass strings again instead of instances, we'd get an error
at query time:

```console
query> allow("Johann", "read", "document-1")
PolarRuntimeError
...
Application error: 'str' object has no attribute 'admin'
```

To guard against this, we can add explicit type checks using
the Polar `matches` operator:

```polar
# A user that is an administrator may read any document.
allow(user, "read", document) if
    actor matches User and
    user.admin = true;

# A user may read any document that they own.
allow(user, "read", document) if
    user matches User and
    document matches Document and
    user.id = document.owner;
```

The `matches` operator succeeds when its left-hand side is an
instance of the type on its right-hand side. For our example
query above, since a `str` is not an instance of either `User`
or `Document`, the `matches` would fail, causing Polar to abort
the query before attempting to access any non-existent fields.

Here `User` and `Document` refer to the application classes defined
above. But in order for Polar to use a class in a type check, it must
first be **registered**. If you're using an ORM adapter, this happens
automatically for all model classes; otherwise, you can register
classes manually using `Oso.register_class`. If you forget, Polar
will warn you about an "unknown specializer" at load time.

### Specializers

Since type checks are common and helpful for rule parameters,
Polar provides a shortcut syntax:

```polar
# A user that is an administrator may read any document.
allow(user: User, "read", document: Document) if
    user.admin = true;

# A user may read any document that they own.
allow(user: User, "read", document: Document) if
    user.id = document.owner;
```

These rules mean the same thing as the ones above, but their
first and third parameters are **specialized** on the `User`
and `Document` classes, respectively: they will only match
when the supplied arguments are instances of (subclasses of)
those classes.

It's considered good practice to use a specializer on any
parameter whose fields you access.

#### Specializers with Fields

Specializers can be used for more than simple type checks.
If the class name is immediately followed by a dictionary
`{field: value, ...}`, then the specializer will only
match the argument if it is of the correct type _and_ all
of the specified fields are present and their values match.
Field matching is done by unification, so may bind variables
to field values. For example, we could rewrite our rules
above as:

```polar
# A user that is an administrator may read any document.
allow(user: User{admin: true}, "read", document: Document);

# A user may read any document that they own.
allow(user: User{id: user_id}, "read", document: Document{owner: document_owner}) if
    user_id = document_owner;
```

Notice that we've dropped the body from the first rule, since
the required condition is now handled entirely by the specializer,
by matching the value of `user.admin` against the literal `true`.

In the second rule, the first specializer binds the variable
`user_id` to the value of `user.id`, and the second binds
`document_owner` to that of `document.owner`. The unification
in the body checks that those two values are equal. But we can
drop that, too, by using the same variable in both specializers:

```polar
# A user may read any document that they own.
allow(user: User{id: user_id}, "read", document: Document{owner: user_id});
```

The first specializer binds `user_id` to `user.id`. Then, since
`user_id` is already bound, the second specializer compares its
value against that of `document.owner`, making the explicit
unification unnecessary.

### Built-in Classes

We said above that any class you use as a specializer
(or on the right-hand side of the `matches` operator) must be
registered with Polar. For convenience, Polar automatically
registers a few **built-in** classes, such as:

* `Dictionary`
* `String`
* `List`
* `Integer`
* `Float`

These classes correspond to ones in the application language,
e.g., in Python, `String` is actually the built-in `str` class.

## Method Calls

In addition to accessing the fields of application objects in a rule,
you can also call methods on them. We'll demonstrate this using the
built-in `String` class, but it applies to instances of any registered
class.

Suppose we wanted to match a range of actions that all started
with the letter "r". (A silly encoding, perhaps, but illustrative.)
We could allow all such actions using a rule like this:

```polar
allow(actor, action: String, resource) if
    action.startswith("r");
```

This calls the Python method `str.startswith`, guarding
against invalid calls with the `String` specializer.

## Other Rules

So far all of our examples have involved only `allow` rules
with three parameters. That's a natural place to start with Oso,
because the `is_allowed` function generates queries of the form
`allow(actor, action, resource)`. But you can write rules for
whatever you like, and use them from within your `allow` rules.
For instance a rule that handles roles might look like this:

```polar
allow(user, action, resource) if
    resource_role_applies_to(resource, role_resource) and
    user_in_role(user, role, role_resource) and
    role_allow(role, action, resource);
```

All three parameters are variables, so they are [bound](#bindings)
to any three arguments. Then queries for each condition in the body
must also succeed, so the rules `resource_role_applies_to`,
`user_in_role`, and `role_allow` must also be defined and match
the supplied arguments.

Rules may also be recursive, i.e., may refer to themselves.
But be sure to define a base case, or queries may loop until
they time out.

Finally, you can query arbitrary rules directly (i.e., without
going through `oso.is_allowed`) by using the `oso.query_rule`
method. This gives you direct access to the knowledge base,
and lets you receive result bindings and continue searching
for results past the first one (`oso.is_allowed` stops after
the first result, since any one valid authorization is as good
as several). It lets you use Oso as a general purpose rule
engine — what other kinds of rules does your application need?

## Summary

In this tutorial, we've seen that:

* A policy in Oso is written as a set of Polar rule definitions.
* Authorization decisions are made by evaluating a query of the
  form `allow(actor, action, resource)` with respect to a policy.
* Rules have names and parameters that may be matched by a query.
* Rules may have bodies, which are conditions that must also be
  true for a query to successfully match the rule.
* Variables may be bound to values during matching, or matched by
  value if they are already bound.
* Rules may refer to application classes, instances, fields, and methods.
* Rules may refer to other rules, including themselves.

{{% callout "What's next" "blue" %}}

- Check out our [How-To Guides](guides) for more on using Polar
  policies for authorization.
- Check out the [Polar reference](reference/polar) for more on the Polar
  language and syntax.

{{% /callout %}}
