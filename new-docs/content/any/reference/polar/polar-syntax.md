---
title: Polar Syntax
any: true
weight: 1
aliases:
  - ../using/polar-syntax.html
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
[Application Types](getting-started/policies#application-types) for more information.

An instance literal can only be used with [the `new` operator](#new) or as a
[pattern](#patterns).

## Rules

Every statement in a Polar file is part of a rule. Rules allow us to express
conditional statements ("**if** this **then** that").

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
corresponding elements unify.:warn

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
    employee(company("Oso"), person(first, last));
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

#### Numerical Comparison

The typical numerical comparison operators can be used to compare values (`> >= < <= == !=`). For example…

```polar
age < 10
```

…will check that the value of the variable `age` is less than 10.

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
(See [Application Types](getting-started/policies#application-types) for more about how to define and
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

For more on this feature, see [Application Types](getting-started/policies#application-types).

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

### Inline Queries (`?=`)

Queries can also be added to Polar files and will run when the file is loaded.
Inline queries can be useful for testing a policy and confirming it behaves as
expected.

To add an inline query to a Polar file, use the `?=` operator:

```polar
?= allow("foo", "read", "bar")
```

An inline query is only valid at the beginning of a line.
