==================
Polar Fundamentals
==================

The **oso** authorization library uses the **Polar** programming language
to express authorization logic and policies. This guide is meant to serve
as a gentle introduction to Polar: what it is, how it works, and what you
can do with it. It is not meant to be an exhaustive or precise description
of the language; instead, its primary goal is to help you understand the
Polar language and its capabilities so that you can start writing policies
quickly and easily.

But before we dive into the specifics, we'll briefly discuss how Polar is
and is not like "traditional" (i.e., imperative) programming languages,
along with some of the advantages and disadvantages that come with those
differences.

Logic Programming
=================
Polar is a `logic programming language
<https://en.wikipedia.org/wiki/Logic_programming>`_,
a paradigm that arose out of early AI research in logical
deduction and planning. In contrast with traditional languages,
a program written in a logic programming language encodes logical
statements about the domain of interest rather than instructions
for carrying out specific operations in that domain. That is,
it uses a *declarative* rather than *imperative* programming
style; you express *what* your program must do, rather than
*how* it must do it. A familiar example of a widely used declarative
language is SQL; others include purely functional languages such
as Haskell, regular expressions, configuration languages, etc.

More concretely, a logic programming language replaces explicit
flow control and algorithmic steps (e.g., loops, conditionals,
function calls, etc.) with *facts* and *rules* that must hold
over your domain objects. These facts and rules are stored in
a special kind of database called a *knowledge base*, so called
because it stores logical statements that represent "knowledge"
(or something roughly analogous to it) about your domain.
As with a traditional database, one does not "run" the knowledge
base; it is not an executable artifact. Instead, one *queries*
it for the truth or falsity of a particular logical statement.

Let's take a simple example. Suppose I tell you that I have a
younger brother and an older sister. Your "knowledge base"
for this example then consists of these two facts:

1. My brother is younger than me.
2. My sister is older than me.

Now I can ask yes/no questions whose answers depend on these
facts. I can ask easy questions like: "is my brother younger
than me?" Such questions are easy because their answers are
directly encoded in the knowledge base: (1) trivially implies
that the answer is "yes". But I can also ask slightly harder
questions, such as: "is my brother older than me?" You can
immediately answer "no", because fact (1) together with the
anti-symmetric nature of the "older than" relation imply that
it is impossible for my brother to be both older and younger
than me. And I can ask questions that are harder still, like:
"is my brother younger than my sister?" By purely logical deduction,
using only basic properties of the relations "older than" and
"younger than" (viz., transitivity), you can answer "yes"
*even though I did not give an explicit relation between the
ages of my siblings*. This simple example almost completely
captures the essence of any logic programming language.

In the context of authorization, the knowledge base represents
your *policy*. It is comprised of facts about and rules governing
the actors, resources, and actions that actors may take on those
resources in your domain. You can then *query* the knowledge
base with a question like: "is this particular actor allowed to
take this action on this resource"? By using the information in
the knowledge base together with a built-in logical deduction process,
the system can answer such queries with either "yes" or "no"
*even when the answer does not appear explicitly in the knowledge base*.
This is the essence of the oso authorization system.

Advantages
----------
One of the main advantages of a purely declarative language
like Polar over a traditional imperative language is *concision*.
This means more than just saving a few characters in typing your
program. It means that you can dramatically *compress* your program
by leveraging a specialized interpreter or search engine.
Another way to think of this is that such languages allow you to
express only and exactly the conditions your problem depends on,
and leave the "incidentals" of *how* or *whether* those conditions
are satisfied to the system.

Take ordinary string-matching regular expressions, for instance.
They offer a level of concision for many simple (and not-so-simple)
string matching tasks that is unmatched by imperative techniques.
Think about the last non-trivial regular expression you wrote;
now think about how you would write an equivalent matching function
in an ordinary imperative language (without writing a regexp interpreter).
Their strength comes not from the raw character count, or their typically
terse choice of metacharacters: a regular expression language that
used, say, ``{ANY}`` instead of ``.`` and ``{END}`` instead of
``$``, etc. would only linearly scale the average pattern length.
Rewriting such expressions as imperative matching statements would
often explode them by a large non-linear factor.

The power and concision of regular expressions for pattern matching
comes from their purely declarative semantics. A regular expression
denotes a set of strings, and the process of matching an input string
is equivalent to searching for that input in the denoted set.
But you don't need to specify *how* the search is performed; the
search algorithm (e.g., NFA, DFA, etc.) is abstracted away from
the pattern language. You don't write an NFA simulator alongside
your regexp, you just assume the interpreter supplies one (or an
equivalent).

Caveats
-------

  Some people, when confronted with a problem,
  think "I know, I'll use regular expressions."
  Now they have two problems.
  — `JWZ <https://www.jwz.org/hacks/>`_

As anyone who's used regular expressions extensively knows,
they can be easier to write than to read. This is because while
you're writing one, the necessary context—the constraints on
the set of strings you want to match—is already in your mind.
But although the regexp encodes those constraints, they can be
difficult to decode if you don't already know them. This is the
flip side of any highly compressed language: because each symbol
potentially carries a large amount of information, it can be
difficult for a human to decode. This is true of any information-dense
notation, e.g., mathematics; without a complete "dictionary"
for the compression scheme, the meaning of statements in an extremely
concise languages can be quite opaque. The flip side of this
flip side, of course, is that *if* the necessary context is
available (e.g., the program or paper is read by someone familiar
with the domain and the notation), vastly more information can
be conveyed in a given space than would otherwise be possible.

There's weight hanging on that *if*, though. Regular expressions
can look like noise to someone unfamiliar with the basic notation.
Logic programs don't usually look like noise, but they might not
look like they *do* much if you don't understand the basic ideas.
And even if you do, they may be sufficiently information-dense
that it can be hard to get details without close scrutiny. The
other side of that point is that while details might be missed,
it's often vastly easier to grasp the program (or policy) as a
*whole*, if only because it might fit on a single page or screen
instead of being spread out over many times that. The old adage
that "a picture is worth a thousand words" neatly captures the
idea here: a statement in a concise declarative language is like
a "picture" of the solution you seek, as opposed to a long-hand
description of how to find it.

Like any picture, though, the "negative space" of a statement
in a declarative language is as important as what is said; i.e.,
what is *implicit* in the meaning of the terms of the language.
To take one last regular expression example, the meaning of the
"any" metacharacter (usually ``.``) depends implicitly on the
space of characters supported by the particular implementation;
e.g., does it match any character that can be encoded in 8 bits,
say, or does it include all Unicode code-points? In logic programming,
this issue manifests itself as the `closed-world assumption
<https://en.wikipedia.org/wiki/Closed-world_assumption>`_:
the assumption that any statement not known to be true is false.
These implicit limitations must be kept in mind both for reading
and writing programs, for they will not appear in the textual
representation.

Polar
=====

Enough abstract nonsense—let's see some code! We'll start, as is
traditional in logic programming, with a simple genealogy example.
Suppose we are given the following fragment of a family tree:

.. image:: /language/olympians.svg

We could represent the direct relations as the following facts in Polar::

  # father(x, y) ⇒ y is the father of x.
  father("Artemis", "Zeus");
  father("Apollo", "Zeus");
  father("Asclepius", "Apollo");
  father("Aeacus", "Apollo");

  # mother(x, y) ⇒ y is the mother of x.
  mother("Apollo", "Leto");
  mother("Artemis", "Leto");

First, some quick syntactic notes. Lines that begin with a ``#`` are
comment lines, and are ignored; they may be used to document your program.
All of the other lines are terminated with a ``;`` to signify the end of
a statement. Double-quoted strings like ``"Artemis"`` and ``"Apollo"``
are literals, and represent the "actors" in our little domain.

Each nontrivial line expresses a **fact**: an unconditionally true
statement in our domain. They collectively define two **predicates**,
``father`` and ``mother``. A predicate is a relation that is either
true or false for a certain set of **arguments**, e.g.,
``father("Artemis", "Zeus")`` (true) or
``father("Zeus", "Zeus")`` (false).

To determine whether a predicate is true or false with respect to a
particular knowledge base, we can **query** it from the interactive
:ref:`REPL <repl>`::

  >> father("Artemis", "Zeus");
  True

Here ``>>`` is the REPL prompt, and the query follows, terminated
with a ``;``. Polar executes the query, and replies ``True``,
since by first fact above, the father of Artemis is indeed Zeus.

That's a fairly trivial query, since its truth value was supplied
directly as a fact. So let's try a non-trivial one: let's ask for
*all* of the known children of Zeus::

  >> father(child, "Zeus");
  child = "Artemis"
  child = "Apollo"
  True

In this query, we used a **variable**, ``child``. Notice that we did
not explicitly assign a value to it; instead, the system *found*
two valid bindings: to the string ``"Artemis"``, and to the string
``"Apollo"``. It did so by **searching** its knowledge base for
facts that **match** the query. We'll dive into the details of this
search process shortly, but let's continue our example for now.
As you might guess, the same sorts of queries work for our other
predicate; if we wanted to know who Artemis's mother was, we could
query for::

  >> mother("Atemis", mother)
  mother = "Leto"
  True

Notice that there is no problem having both a variable and a predicate
named ``mother``. In Polar, variables cannot be bound to predicates
(it is a `first-order logic language <https://en.wikipedia.org/wiki/First-order_logic>`_),
so they use different namespaces.

Now let's augment our simple facts with some **rules**. Rules are
like facts, but conditional—they define relations that are true
**if** some other conditions hold. Rules are strictly more general
than facts, since any fact is simply a rule with no conditions.
As with facts, multiple rules may be defined for the same predicate,
and conversely a predicate may be defined by any mixture of facts
or rules. Here's a rule that we could define::

  # parent(x, y) ⇒ y is a parent of x.
  parent(x, y) if father(x, y);
  parent(x, y) if mother(x, y);

Again, let's start with the syntax. Each rule has a **head** and
a **body**, separated by the ``if`` symbol. (If there is no body,
the ``if`` is elided, and the rule becomes a fact.) To apply
a rule, Polar first matches the head with the query (just as
for a fact), and then queries for the body. If that sub-query
is successful, then the rule as a whole succeeds; otherwise,
it fails and tries the next one. Thus, multiple rules for the
same predicate are *alternatives*: ``y`` is a parent of ``x``
*if* either ``y`` is the mother of ``x`` *or* the father of
``x``. Let's see it in action::

  >> parent("Apollo", "Zeus");
  True
  >> parent("Apollo", "Leto");
  True
  >> parent("Apollo", "Artemis");
  False
  >> parent("Artemis", parent);
  parent = "Zeus"
  parent = "Leto"
  True

We can go one level deeper, if we wish::

  # grandfather(x, y) ⇒ y is a grandfather of x.
  grandfather(x, y) if parent(x, p) and father(p, y);

This rule has two conditions in its body, separated by the
conjunction operator ``and``. It says that ``y``
is a grandfather of ``x`` *if* there is some ``p`` that
is the parent of ``x`` *and* ``y`` is the father of that
``p``. For example::

  >> grandfather("Asclepius", g);
  g = "Zeus"
  True

We can also write recursive rules::

  # ancestor(x, y) ⇒ y is an ancestor of x.
  ancestor(x, y) if parent(x, y);
  ancestor(x, y) if parent(x, p) and ancestor(p, y);

This says that ``y`` is an ancestor of ``x`` *if* ``y`` is either a
parent of ``x`` *or* they are an ancestor of a parent ``p`` of ``x``::

  >> ancestor("Asclepius", ancestor);
  ancestor = "Apollo"
  ancestor = "Zeus"
  ancestor = "Leto"
  True

The Search Procedure
--------------------

Now that we've seen some basic examples of Polar rules and queries,
let's look in a little more detail at how it executes queries against
a given set of rules.

Recall that rules have a **head** and an optional **body** (the part
after a ``if``). If there is no body, we call the rule a **fact**. The head
must contain exactly one predicate, with any number of **parameters**
in parenthesis; e.g., ``1`` is not a valid head, nor is a bare ``foo``.
Unlike most non-logic languages, each parameter may be either a variable
*or* a constant (literal); e.g., ``foo(1)``, ``foo("foo")``, and ``foo(x)``
are all perfectly good rule heads.

You may also define rules with the same name but a different number of
parameters; e.g.::

  foo(1);
  foo(1, 2);

Semantically, this actually defines two *different* predicates, which are
traditionally written as ``foo/1`` and ``foo/2``. We don't use predicates
overloaded in this way very often, but they are occasionally useful, e.g.,
one could be a "public" predicate, and the other a recursive helper that
also takes, say, an accumulator or something like that. (There is no
visibility control in Polar, so the helper wouldn't really be "private",
it would just never be queried directly except by the "public" predicate.)

But let's keep things simple for now. Suppose our knowledge base
consists of just these two facts::

  foo(1);
  foo(2);

Now consider the query ``foo(2)``. Polar first looks up all of the
rules for the predicate ``foo``, and finds the two above. Then, for
each of those rules, it tries to match each argument of the query
with the corresponding parameter in the head of the rule. In this
case, the argument is ``2``, and the corresponding first parameter
of the first rule is ``1``, so the match fails. But matching with
the head of the second rule succeeds, since ``2 = 2``. There is no
body for this rule, so the match is unconditional, and the query
succeeds: ``foo(2)`` is true.

Now let's consider a slightly more complex query: ``foo(x)``. Once
again, the two rules above are considered. But now they *both* match,
because ``x = 1`` and ``x = 2`` are valid **bindings** for ``x``
(though not at the same time). The equals sign ``=`` in Polar is thus
not quite a comparison operator, but not quite assignment, either—it's
sort of a mixture of both. It's called a
`unification <https://en.wikipedia.org/wiki/Unification_(computer_science)>`_
operator, and it works like this: if either side is an unbound variable,
it is **bound** to the other side, and the result is true; otherwise, the
two sides are compared for equality (element- or field-wise for compound
value types like lists and dictionaries), with variables replaced by
their values. For example, (even without any rules) the conjunctive
query ``x = 1 and x = 1`` succeeds, because the first unification binds
the variable ``x`` to the value ``1``, so the second unification
is equivalent to ``1 = 1``, which is true. But the query ``x = 1 and x = 2``
is false, because the second unification is equivalent to ``1 = 2``.

We can now state precisely how the search procedure works for predicates.
For each rule defined on the query predicate, each query argument is
unified, in left-to-right order, with the corresponding parameter in
the head of the rule. That unification may or may not bind variables,
either from the parameter or the argument. Each unification occurs in
a dynamic environment that contains the bindings from the previous
unifications. If all of the unifications of the query arguments with
the parameters in the head succeed, then a sub-query for the body
of the rule is executed. The body of a rule may consist of a single
predicate, or a conjunction of them, or of any of the operators
described in the :doc:`Polar language reference </language/index>`,
e.g., disjunction, negation, numeric comparisons, etc. Each conjunct
is queried for, in left-to-right order, accumulating any bindings
from unifications. If the queries for every conjunct in the body all
succeed, then the query as a whole does, too.

When a top-level query succeeds, Polar pauses and reports the set
of bindings (which may be empty) that enabled the successful query;
this is what we saw in the REPL examples above. It then *continues*
searching for more solutions, picking up with the next matching rule.
Thus, Polar searches for all possible bindings that make the query
predicate true in the space determined by the rules in the knowledge
base, just as a regular expression pattern match searches for a query
string in the set of strings determined by the regexp.

If a query fails (i.e., is false), then the current branch of the search
is abandoned, and Polar **backtracks** to the last alternative, which
will be either another possible rule for the current query, or the next
untaken branch of a disjunction. When backtracking, all variable bindings
that occurred since the last alternative are undone. If no unexplored
alternatives remain, the query as a whole fails, and a false result
is reported.

.. todo::
   a graphical representation of the search procedure.

Summary
=======
In this guide, we have explored:

* Declarative programming, and logic programming in particular.
* The basic syntax and semantics of the Polar language.
* The search and unification procedures that Polar is built on.

What we haven't explored here is how to use Polar to express
particular authorization policies. Many examples can be found
in the :doc:`Authorization Fundamentals </auth-fundamentals>`
and :doc:`Authorization Models </auth-models/index>` sections
of the manual.
