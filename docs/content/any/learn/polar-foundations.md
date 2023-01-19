---
date: '2021-12-02T02:46:33.217Z'
title: The Polar Language
aliases:
    - ../../more/language/polar-foundations.html
weight: 4
---

# The Polar Language

The Oso authorization library uses the Polar programming language to express authorization logic and policies. This guide is an introduction to the Polar language: what it is, how it works, why we've chosen to use it. For a more complete guide to syntax, use our [Syntax Guide](reference/polar/polar-syntax).

## Declarative Programming

Polar is a [declarative programming language](https://en.wikipedia.org/wiki/Declarative_programming). It's very different from the *imperative* programming languages—like Python, JavaScript, or Go—that we often do our day-to-day work in.

In a declarative programming language, you state what your program should do, and the language runtime will compute it.

- SQL is a declarative programming language: you state what records you'd like to fetch, and the SQL runtime determines what steps to take to return those records.
- Regular expressions are a declarative language: you write what patterns you'd like to be matched, and it's up to the runtime to return the text that matches those patterns.

Polar is similar. You'll write authorization *rules*, query those rules, and Polar will tell you what your query matched. Working with Polar is much like working with a database. When writing code, you'll write information to your database. At runtime, you'll query that information.

## Logic Programming

Polar is also a [logic programming language](https://en.wikipedia.org/wiki/Logic_programming). This means that it's designed to answer questions about a set of rules. You'll see how this works in the *How Polar code executes* section.

## Advantages

- Declarative languages like Polar are concise. This means more than just saving a few characters in typing your program. You can dramatically *compress* your program by leveraging the language runtime. Being able to express your program concisely means simpler programs, fewer places to make mistakes, and less complexity when you're making changes.
- Logic programming is very well-suited to the domain of authorization.  Authorization queries like, "Is this user allowed access to this resource?" are easy to answer with a logic programming language.

## Caveats

- It takes practice to read Polar code. If you've used regular expressions extensively, you know that it takes some practice to look at a regular expression and see what it does. Polar is similar—at first, it looks like Polar statements aren't doing much. That's because the language runtime handles so much for us.
- Polar executes in a way that might be unfamiliar to you. It runs very differently from how most app code executes. That's why we have these guides—we'll help you get fluent in Polar!

## How Polar code executes

> For the next few examples, we'll use only the base language, without touching authorization just yet.
>

Here's one Polar rule.

```polar
father("Artemis", "Zeus");
```

In words, this line means "`father` is true when it's called on the strings `"Artemis"` and `"Zeus"`." This short example defines a rule named `father`. It does this without an explicit definition step! No need to write `def father():`.

We can add another rule:

```polar
father("Artemis", "Zeus");
father("Apollo", "Zeus");
```

These lines mean:

- "`father` is true when it's called on the strings `"Artemis"` and `"Zeus"`."
- "`father` is *also* true when it's called on the strings `"Apollo"` and `"Zeus"`."

Notice that these rules exist side-by-side. We can have any number of rules that use the `father` predicate—adding a new rule is much like adding a new database entry.

Now that we've written these rules, we can *query* them. We'll need to run the program to query these rules. The easiest way to do that is to run the interactive REPL:

```polar
$ python -m polar father.polar
>> father("Artemis", "Zeus");
True
```

We asked a question about the program, and got our answer: "`father` is true when it's called on the strings `"Artemis"` and `"Zeus"`." (We already knew that, though—that was what the rule meant.)

Let's ask a more open-ended question.

```polar
>> father(child, "Zeus");
child = "Artemis"
child = "Apollo"
True
```

This asks, "what are all the values, called `child`, for which `father(child, "Zeus")` is true?" And we get an answer: `child` could be either "Artemis" or "Apollo". The word `child` isn't special—any word that's not already defined becomes a *variable*, and Polar will look for all values that variable could be.

**Conditional rules**

So far, we've seen rules that are simply true. We can also write rules that are conditionally true. Here's one:

```polar
grandfather(a, b) if father(a, anyPerson) and father(anyPerson, b);
```

Like we saw above, we can use an unused word—in this case, `anyPerson`—and that word functions as a variable.
To use this rule effectively, we'll need one more `father` rule:

```polar
father("Artemis", "Zeus");
father("Apollo", "Zeus");
father("Asclepius", "Apollo");
grandfather(a, b) if father(a, anyPerson) and father(anyPerson, b);
```

Now, we can ask our programs questions about this rule.

```polar
>> grandfather("Asclepius", grandpa);
grandpa = "Zeus"
True
```

Our program has deduced the grandfather of Asclepius!

Most Polar rules you'll see are in this `statement if condition;` form. That's where we'll wrap up this guide—to dive deeper into Polar syntax, we have a Polar **[Syntax Guide](reference/polar/polar-syntax)**.

We haven’t covered how to use Polar to express particular authorization policies. Many Polar examples can be found in our [authorization guides](https://docs.osohq.com/guides.html).
