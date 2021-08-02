---
title: REPL
aliases:
    - ../more/dev-tools/repl.html
description: Oso provides a simple REPL (Read, Evaluate, Print Loop) to interactively query a policy.
---

# REPL

The usual way to query an Oso knowledge base is through the API in your
application’s language. But especially during development and debugging,
it can be useful to interactively query a policy. So Oso provides
a simple REPL (Read, Evaluate, Print, Loop). To run it, first make sure
that you have installed Oso.

Once Oso is installed, launch the REPL from the terminal:


{{% exampleGet startRepl %}}

At the `query>` prompt, type a Polar expression and press `Enter`.
The system responds with an answer, then prints the `query>` prompt
again, allowing an interactive dialog:

```
query> 1 = 1
true
query> 1 = 2
false
query> x = 1 and y = 2
y => 2
x => 1
query> x = 1 or x = 2
x => 1
x => 2
query> x = 1 and x = 2
false
```

If the query can not be satisfied with the current knowledge base,
the response is `false`. If the query is unconditionally true, then
the response is `true`. Otherwise, each set of bindings that *makes*
it true is printed; e.g., the third example above has one such set,
the fourth has two.

To exit the REPL, type `Ctrl-D` (EOF).

## Loading Policy and Application Code

To query for predicates defined in a policy, we’ll need to load the
policy files. For instance, suppose we had just one `allow` rule for
Alice, say, in the file `alice.polar`:

```
allow("alice@example.com", "GET", expense: Dictionary) if
    expense.id == 1;
```

Then we can run the REPL, passing that filename (and any others we need)
on the command line:

{{% exampleGet startReplWithFile %}}

And now we can use the rule that was loaded:

```
query> allow("alice@example.com", "GET", {name: "my expense", id: 1})
true
```

We can also use application objects in the REPL, but we have to load
and register the defining modules before we launch the REPL. The easiest
way to do that is to write a script that imports the necessary modules,
plus `oso`, and then use the {{% apiDeepLink module="oso" class="Oso" %}}repl{{% /apiDeepLink %}}
API method to start the REPL:

{{% exampleGet replApi %}}

