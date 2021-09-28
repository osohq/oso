---
title: Polar Syntax
any: true
weight: 1
aliases:
  - ../../using/polar-syntax.html
description: |
  A brief description of the core syntax elements of Polar.
---

# Polar Syntax

Polar is a declarative logic programming language, specialized for making
authorization decisions and tightly integrating with your application’s native
language.

This guide is a brief description of the core syntax elements of Polar.

Each Polar file defines a set of rules. When a Polar file is loaded into the
authorization engine, all rules are added to the engine’s knowledge base.

The knowledge base may be queried. The behavior of queries is described further
[here](learn/polar-foundations#the-search-procedure).

## Primitive Types

Polar has only a few primitive data types.

### Numbers

Polar parses unquoted integers or floating point numbers as numeric values. For
example, all of the following are parsed as numbers:

```polar
22
43
-7
22.3
-22.31
2.0e9
```

You can also perform basic arithmetic on numbers with the operators `+`, `-`,
`*`, `/`, `mod`, and `rem`.

### Boolean

Polar parses the keywords `true` and `false` as boolean values.

### Strings

Polar supports quoted strings, which can be used to represent any textual data.
Polar strings are quoted with double quotes (`"`). Quotes within strings can be
escaped with a single backslash. Two strings are considered equal if they have
the same length and each of their corresponding characters are equal.

The string type can be referenced (for use in specializers, or with the
`matches` operator) as `String`.

## Compound Types

To support more complex data, Polar includes the following compound data types.

### Lists

A list is a sequence of values defined using brackets: `[v1, v2, ..., vn]`. For
example:

```polar
["polar", "lang", "oso"]
["oso", ["polar", "lang"]]
```

Lists may have any length. List membership can be determined using [the `in`
operator](#in-list-membership).

### Dictionaries

While lists are useful for representing ordered data, dictionaries (sometimes
known as hash tables or associative arrays) can express unordered relational
data such as mappings: `{key1: value1, key2: value2, ..., keyN: valueN}`.

For example:

```polar
{first_name: "Yogi", last_name: "Bear"}
```

### Class Instances

A similar syntax is used to represent instances of classes. The class name is
specified before the dictionary:

```polar
Bear{first_name: "Yogi", last_name: "Bear"}
```

Classes can be registered with the Oso library to integrate with Polar. See
[Application Types](guides/policies#instances-and-fields) for more information.

An instance literal can only be used with [the `new` operator](#new) or as a
[pattern](#patterns).

## Rules

Rules allow you to express conditional statements ("**if** this **then** that").

<!-- TODO: this is not a great explanation as it uses the term "fact" which we don't explain. -->
A rule in Polar takes the form `HEAD if BODY;` where `HEAD` must be a _fact_
and `BODY` any number of _terms_. The meaning of a rule is that `HEAD` is true
**if** each of the `BODY` terms is true. If there are be multiple rules with
the same head, each `BODY` will be tried in turn, and any or all may succeed.
For more on how rules are defined and applied see [Polar
Background](learn/polar-foundations).

The following is an example of a rule:

```polar
person("yogi", "bear") if bear("yogi", "bear");
```

This example says that Yogi is a person **if** Yogi is a bear. Bears are
people, too.

### Terms

A _term_ is either a data type or a combination of facts using
[operators](#operators).

### Variables

The example rule above is static. More powerful rules can be formed using
variables. In Polar, a variable does not need a separate declaration; it is
created the first time it is referenced. Variables can be substituted for
values in dictionaries or items in a list or rule call.

The following are all variables:

```polar
foo
bar
myvar
```

To make the above rule more useful, we could write:

```polar
person(first, last) if bear(first, last);
```

This rule says that **if** there is a bear with some name, **then** that bear
is also a person.

#### Singletons

If a variable occurs only once, then its value can’t be used for anything. Such
variables are called _singletons_, and Polar will warn you if they occur in a
rule. For example, if you try to load the rule…

```polar
user(first, last) if person("George", last);
```

…you'll see the following message:

```console
Singleton variable first is unused or undefined
001: user(first, last) if person("George", last);
          ^
```

The reason these warnings are important is that, as in this case, they indicate
potential logical errors. Here, the error is forgetting to use the first name,
and instead using a literal string in the call to `person`.

There are cases, however, where it _isn’t_ an error to have a singleton
variable. For example:

- As a parameter with a specializer: `allow(_actor: Person{first_name: "George"}, …);`
- As a parameter that is explicitly ignored: `always_true(_);`

In such cases, you can suppress the singleton variable warning by starting your
variable’s name with an `_` (underscore), e.g., `_actor` in the first example
above.

A variable named _just_ `_` (as in the second example above) is called an
**anonymous** variable, and it is _always_ a singleton (but will never generate
a warning). Each occurrence is translated into a fresh variable, guaranteed not
to match any other variable. You may therefore have as many anonymous variables
in a rule as you like, and each will be unique. It’s up to you whether to use
an anonymous variable or a singleton with a descriptive name.

### Operators

Operators are used to combine terms in rule bodies into expressions.

#### Unification

Unification is the basic matching operation in Polar. Two values are said to
_unify_ if they are equal or if there is a consistent set of variable bindings
that makes them equal. Unification is defined recursively over compound types
(e.g., lists and dictionaries): two compound values unify if all of their
corresponding elements unify.

Unification may be performed explicitly with the unification operator (`=`),
which is true if its two operands unify; e.g., `1 = 1`, `"a" = "a"`, or `x = 1`
where the variable `x` is either bound to `1` or unbound.

Unification is also used to determine if queries match rule `HEAD` s, and if
the `BODY` of rules match other facts in the knowledge base. We will cover
unification further in [The Search
Procedure](learn/polar-foundations#the-search-procedure).

#### Assignment

Assigning a value to an unbound variable can be done using the unification operator.
However, the assignment operator (`:=`) may also be used, and will only succeed if the
left-hand side operand is an unbound variable. For example, `foo := 1`.
This operator can be used to improve readability and predictability
by indicating explicit assignment. Attempting to assign to a non-variable will result in a parse error,
while attempting to assign to a bound variable will result in a runtime error.

#### Conjunction (and)

To say that two terms in a rule’s body must **both** be true, the `and`
operator can be used. For example, the rule…

```polar
oso_user(first, last) if
    user(first, last) and
    employee("Oso", first, last);
```

…will be satisfied if the named person is a user **and** that person is an
employee of Oso.

#### Disjunction (or)

The `or` operator will be true if either its left **or** its right operand is
true. Disjunctions can always be replaced by multiple rules with identical
heads but different bodies (the operands), but may help simplify writing rules
with alternatives.

#### Negation (not)

The `not` operator will succeed when its argument fails.

For example, the following rule will succeed when `x != 0` (and could be written as such).

```polar
non_zero(x) if not x == 0;
```

`not` is helpful when negating the results of another rule call. For example,

```polar
positive(x) if
    non_zero(x) and
    not negative(x);

negative(x) if x < 0;
```

Above, `positive` will succeed if `negative(x)` returns no results.

To negate multiple expressions, use parentheses to group them:

```polar
positive(x) if
    not (x == 0 or negative(x));
```

#### Dictionary Key Access

The dot `.` operator can be used to access the value associated with a key in a
dictionary. For example:

```polar
dict = { hello: "world" } and
dict.hello = "world"
```

A string stored in a variable can be used as the key in a dot lookup using the
following syntax:

```polar
dict = { hello: "world" } and
key = "hello" and
dict.(key) = "world"
```

#### Application Field or Method Access

The dot `.` operator can also be used to access methods or fields on
application instances or constants. Arguments can be passed into methods as
positional or keyword arguments, depending on the application language. Keyword
arguments are only supported in languages that themselves support them (e.g.,
Python, Ruby).

Accessing a field on an application instance looks like:

```polar
person = new Person() and
first_name = person.first_name
```

Calling an application method from a policy looks like this:

```polar
person = new Person() and
person.check_address("12345 Broadway", city: "New York", state: "NY");
```

Calling a class method on a class that has been registered as a constant could
look like:

```polar
person = new Person() and
Person.log("created new person")
```

#### Comparison

The comparison operators can be used to compare values (`> >= < <= == !=`). For example…

```polar
age < 10
```

…will check that the value of the variable `age` is less than 10.
Performing a comparison on application data will use the host language's
native comparison operation. Not all Oso language libraries support this
feature.

#### Print

`print()` is a built-in operator that prints its arguments to the console. It
always succeeds and can therefore be added anywhere in the body of a Polar
rule. For example:

```polar
hello(x) if print("hello", x) and x = "world";

query> hello("world");
"hello", "world"
True
```

#### Cut

By default, Polar runs all of the definitions for a given rule that are
applicable to the given set of arguments (i.e., whose specializers are
matched). The `cut` operator overrides this behavior by _committing_ to the
enclosing rule definition: the query engine will not run any others. Rule
definitions that have already run are not “un-run”, though, or avoided by using
cut; it just ensures that no _others_ will run.

Because Polar runs rules in most-to-least-specific order, these “other” rule
definitions are always _less specific_ than the current one; i.e., they may
have specializers that are superclasses (and therefore less specific) of those
in the current rule. This allows `cut` to override a rule that is specialized
on a less specific class. You can think of `cut` as a sort of dual to `super()`
in other object-oriented languages (e.g., Python): in Polar, the behavior of
“methods” (rules) is to implicitly call the next method, but `cut` overrides
that behavior; it says _not_ to call any more methods (rules).

`cut` can appear anywhere in a rule body, but terms before it must succeed for
it to be reached, so it frequently appears at the end of the body: **if**
so-and-so is true, then **cut** out all other alternatives.

`cut` should be used sparingly.

#### New

The `new` operator is used to construct a new instance of an application class.
(See [Application Types](guides/policies#instances-and-fields) for more about how to define and
register application classes.) The name of the class to instantiate comes next,
followed by a set of initialization arguments that are passed to the class’s
constructor:

```polar
new Bear("yogi", "bear")
```

In host languages that support keyword arguments (e.g., Python & Ruby, but not
Java), you can pass initialization arguments as keywords with the following
syntax:

```polar
new Bear(first_name: "yogi", last_name: "bear")
```

If using a constructor with mixed positional and keyword arguments, positional
arguments must come before keyword arguments:

```polar
new Bear("yogi", last_name: "bear")
```

#### In (List Membership)

The `in` operator can be used to iterate over elements of built-in and
application types. Iterable built-in types are `List`, `String`, and
`Dictionary`.

The first operand will be unified with each element. If the second operand is
not iterable, the operation will fail. For example…

```polar
x in [1, 2, 3] and x = 1
```

…will bind `x` to `1`, `2`, `3`, in turn, and check that `x = 1` for each. This
expression will only succeed for the first item (`1`). The left-hand side does
not need to be a variable. for example…

```polar
1 in [1, 2, 3, 1]
```

…will succeed _twice_: 1 is in the first and fourth position.

Iterating over a `String` returns each character (as another string):

```polar
hexstring(s) if
  forall(c in s, c in "0123456789abcdef");
```

Iterating over a dictionary returns a list with two elements, the key (as a
string) and the value:

```polar
x in {a: 1, b: 2}
[key, _] in {a: 1, b: 2}
[_, value] in {a: 1, b: 2}
```

The above returns:

```polar
x => ["a", 1]
x => ["b", 2]

key => "a"
key => "b"

value => 1
value => 2
```

#### For All

The `forall` operator is often useful in conjunction with the `in` operator.
`forall(condition, action)` checks that `action` succeeds for every alternative
produced by `condition`. For example…

```polar
forall(x in [1, 2, 3], x = 1)
```

…would fail because `x` only unifies with `1` for the first element in the list
(the first alternative of the condition). In contrast…

```polar
forall(x in [1, 1, 1], x = 1)
```

…succeeds because the `action` holds for all values in the list.

`forall` can also be used with application data to check all elements returned
by an application method:

```polar
forall(role = user.roles(), role = "admin")
```

Any bindings made inside a `forall` (`role` or `x` in the example above) cannot
be accessed outside the `forall` operation.

#### `*rest` Operator

The rest operator (`*`) can be used to destructure a list. For example:

```polar
x = [1, 2, 3] and
[first, *tail] = x
```

After evaluating the above, the variable `first` will have the value `1` and
`tail` will have the value `[2, 3]`.

The rest operator is only valid within a list literal and in front of a
variable. It **must** be the last element of the list literal (`[*rest, tail]`)
is invalid. Any number of elements can come before the rest operator.

The rest operator is only useful when combined with a unification operation
that assigns a value to it.

### Patterns and Matching

Polar has powerful pattern matching facilities that are useful to control which
rules execute & in what order.

#### Specialization

Rule heads (the part of the rule before the `if` keyword) can contain
specializers. For example, the rule…

```polar
has_first_name(person: Person, name) if person.name = name;
```

…would only execute if the `person` argument is of the type `Person`.

Multiple rules of the same structure can be written with different
specializers:

```polar
has_first_name(user: User, name) if user.name = name;
```

Now, the `first_name` rule can be used with instances of the `User` or `Person`
type.

For more on this feature, see [Application Types](guides/policies#instances-and-fields).

#### Patterns

The expression after the `:` is called a pattern. The following are valid
patterns:

- any primitive type
- a dictionary literal
- an instance literal (without the new operator)
- a type name (used above)

When a rule is evaluated, the value of the argument is matched against the
pattern. For primitive types, a value matches a pattern if it is equal.

For dictionary types, a value matches a pattern if the pattern is a subset of
the dictionary. For example:

```polar
{x: 1, y: 2} matches {x: 1}
{x: 1, y: 3} matches {y: 3}
{x: 1, y: 3} matches {x: 1, y: 3}

# Does not match because y value are not equal
not {x: 1, y: 3} matches {x: 1, y: 4}
```

A type name matches if the value has the same type:

```polar
new Person() matches Person
```

The fields are checked in the same manner as dictionaries, and the type is
checked in the same manner as the previous example:

```polar
new Person(x: 1, y: 2) matches Person{x: 1}
```

For type matching, subclasses are also considered. So, a class that is a
subclass of `Person` would match `Person{x: 1}`.

#### Matches Operator

The above example used the `matches` operator to describe the behavior of
pattern matching. This operator can be used anywhere within a rule body to
perform a match. The same operation is used by the engine to test whether a
rule argument matches the specializer.

#### Actor and Resource Specializers

Oso provides built-in specializers that will match any
application type that has been declared via an [actor or resource block](#actor-and-resource-blocks).

The `Actor` specializer will match any application type that has been declared via an `actor` block,
and `Resource` will match types declared via `resource` blocks.

E.g., the following is a valid head for an `allow` rule:

```polar
allow(actor: Actor, action, resource: Resource) if ...
```

Because of this, attempts to register application types named `Actor` or
`Resource` will result in an error.

`Actor` and `Resource` specializers are used by Oso's built-in [rule types](#rule-types) to validate policies.

### Inline Queries (`?=`)

Queries can also be added to Polar files and will run when the file is loaded.
Inline queries can be useful for testing a policy and confirming it behaves as
expected.

To add an inline query to a Polar file, use the `?=` operator:

```polar
?= allow("foo", "read", "bar")
```

An inline query is only valid at the beginning of a line.

### Rule Types

A rule type specifies the _shape_ of a rule — its number of arguments and, optionally, the type of each argument. If a rule type exists for `has_permission()`, then all `has_permission()` rules must conform to the rule type.
Rule types have the same syntax as rule heads and are preceded by the keyword `type`:

```polar
type has_permission(actor: Actor, action: String, resource: Resource);
```

The above rule type specifies that any rule with the name `has_permission` must have three arguments where the first argument matches `Actor`, the second argument matches `String`, and the third argument matches `Resource`.

Argument matching is determined in the same way that matching is determined for rule evaluation. See [Patterns and Matching](#patterns-and-matching).

Rule types are optional. If a rule type exists with the same name as a rule, then the rule must match that type or else an error will be thrown when the policy is loaded.
If multiple rule types are defined for the same rule name, then a rule need only match one type to be valid.

You can find a reference for built-in rule types [here](reference/polar/builtin_rule_types).

## Actor and Resource Blocks

Actor and resource blocks provide a way to organize authorization logic by application type.
These blocks are especially useful for expressing role-based access control logic.


The simplest form of a block looks like this:

```polar
# Actor block
actor User {}

# Resource block
resource Repository {}
```

In the above example, `User` and `Repository` must be registered [application types](classes).

Inside of a block, you can declare [permissions](#permission-declarations), [roles](#role-declarations), and [relations](#relation-declarations) and write [shorthand rules](#shorthand-rules).

A more complete block looks like this:

{{< literalInclude dynPath="rolesPolicyPath"
                   from="docs: blocks-start"
                   to="docs: blocks-end" >}}

<!--
TODO: should we add the data-linking rules here too? I'm thinking in case
someone just copies and pastes this whole thing
-->

Once you have declared a block, you can use the built-in [`Actor` and `Resource`
specializers](#actor-and-resource-specializers) to match all types declared as actors or
resources, respectively.

### Permission Declarations

You can specify the permissions that are available for an actor or resource type using the following syntax:


{{< code codeLang="polar" hl_lines="2">}}
resource Repository {
  permissions = ["read", "push"];
}
{{< /code >}}

Permissions are always strings. You must declare permissions in order to use them in [shorthand rules](#shorthand-rules).

### Role Declarations

You can specify the roles that are available for an actor or resource type using the following syntax:

{{< code codeLang="polar" hl_lines="2">}}
resource Repository {
  roles = ["contributor", "maintainer", "admin"];
}
{{< /code >}}

Roles are always strings. You must declare roles in order to use them in [shorthand rules](#shorthand-rules).

In order to use roles, you must write at least one `has_role` rule that gets
user-role assignments stored in your application. This rule takes the following
form:

```polar
has_role(actor: Actor, name: String, resource: Resource) if ...
```

For example:

```polar
# User-role assignment hook - required when using roles
has_role(user: User, name: String, repo: Repository) if
  # Look up user-role assignments from application, e.g.
  role in user.roles and
  role.repo_id = repo.id;
```

The `name` argument corresponds to the role names in the declaration list. The
`has_role` rule must handle every declared role name, otherwise you may encounter application errors or unexpected policy behavior.
### Relation Declarations

You can specify relations between actor/resource types using the following syntax:

{{< code codeLang="polar" hl_lines="2">}}
resource Repository {
  relations = { parent: Organization };
}
{{< /code >}}

Relations are `key: value` pairs where the key is the relation name and the value is the type of the related object.
Related object types must also be declared in resource or actor blocks.

In order to use relations, you must write a `has_relation` rule that gets relationship data from your application. This rule takes the following form:

```polar
has_relation(subject: Resource/Actor, name: String, object: Resource/Actor) if ...
```

The `object` argument is the resource or actor type on which the relation was declared.
In the example above, the object type is `Repository` and the subject type is `Organization`.

For example:

```polar
# Relation hook - required when using relations
has_relation(parent_org: Organization, "parent", repo: Repository) if
  # Look up parent-child relation from application, e.g.
  parent_org = repo.organization;
```

`has_relation` rules must be defined for every declared relation.

### Shorthand Rules

Shorthand rules are concise rules that you can define inside actor and
resource blocks using declared permissions, roles, and relations.

For example,

{{< literalInclude dynPath="rolesPolicyPath"
                   from="docs: blocks-start"
                   to="docs: blocks-end" >}}

For shorthand rules to be evaluated by the Oso library, you must add the following rule to your policy:

```polar
allow(actor, action, resource) if has_permission(actor, action, resource);
```

This rule tells Oso to look for permissions that were granted through shorthand rules.

#### Shorthand Rules Without Relations

A shorthand rule has the basic form:

```polar
[result] if [condition];
```

Where `"result"` and `"condition"` can be [permissions](#permission-declarations) or [roles](#role-declarations) that were declared inside the same block.

For example:

```polar
resource Repository {
  permissions = ["read", "push"];
  roles = ["contributor", "maintainer"];

  "read" if "contributor";  # "contributor" role grants "read" permission
  "push" if "maintainer";  # "maintainer" role grants "push" permission
  "contributor" if "maintainer";  # "maintainer" role grants "contributor" role
}
```

#### Shorthand Rules With Relations

If you have [declared relations](#relation-declarations) inside a block, you can also write shorthand rules of this form:

```polar
[result] if [condition] on [relation];
```

where `result` and `condition` can be permissions or roles, and `relation` can be a relation.

This form is used to grant results based on conditions on a related resource or
actor. This form is commonly used with `"parent"` relations.

For example,

```polar
resource Repository {
  roles = ["contributor", "maintainer"];
  relations = { parent: Organization };

  "admin" if "owner" on "parent"  # "owner" role on parent Organization grants the "admin" role
  "contributor" if "member" on "parent"  # "member" role on parent Organization grants "contributor" role
}
```

### Shorthand Rule Expansion

Shorthand rules are expanded to full Polar rules when they are loaded. The semantics of this expansion are as follows.

#### Expansion without relation

```polar
$x if $y;
=> rule1(actor: Actor, $x, resource: $Type) if rule2(actor, $y, resource);
```

where `rule1` and `rule2` are the expansions of `$x` and `$y` respectively.

If `$x` is a [permission](#permission-declarations), then `rule1` will be
`has_permission`. If `$x` is a [role](#role-declarations), then `rule1` will be
`has_role`. The same semantics apply for `$y` and `$rule2`.

The resource argument specializer `$Type` is determined by the enclosing [block definition](#actor-and-resource-blocks).
E.g., if the rule is defined inside of `resource Repository {}`, then `$Type` will be `Repository`.

For example,

```polar
# Shorthand rule
resource Repository {
  permissions = ["read"];
  roles = ["contributor"];

  "read" if "contributor";
}

# Expanded rule
#                            "read"                        if                 "contributor"           ;
#                              \/                                                  \/
has_permission(actor: Actor, "read", resource: Repository) if has_role(actor, "contributor", resource);
```


#### Expansion with relation

```polar
$x if $y on $z;
=> rule1(actor: Actor, $x, resource: $Type) if rule2(actor, $y, related) and has_relation(related, $z, resource);
```

where `rule1`, `rule2`, and `has_relation` are the expansions of `$x`, `$y`, and `$z` respectively.

The expansion of `$x` to `rule1` and `$y` to `rule2` follow the same semantics as expansion without relation above.
`$z` must always be a [declared relation](#relation-declarations) in the enclosing block.
The `has_relation` rule is necessary in order to access the `related` object that `rule2` references.
This expansion shows why it is necessary to define `has_relation` rules for every declared relation.


For example:

```polar
resource Repository {
  roles = ["admin"];
  relations = {parent: Organization};

  "admin" if "owner" on "parent";
}

# Expanded rule
#                      "admin"                        if                 "owner"            on                        "parent"          ;
#                        \/                                                \/        /------|-----------------\         \/
has_role(actor: Actor, "admin", resource: Repository) if has_role(actor, "owner", related) and has_relation(related, "parent", resource);
```
