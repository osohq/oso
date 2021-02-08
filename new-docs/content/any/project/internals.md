---
title: Internals
aliases: 
    - ../more/internals.html
---

# Internals

Oso is supported in a number of languages,
but the [Oso core](https://github.com/osohq/oso) is written in Rust,
with bindings for each specific language.

At the core of Oso is the **Polar language**. This handles parsing
policy files and executing queries in the form of a virtual machine. Oso was
designed from the outset to be natively embedded in different
languages. It exposes a foreign function interface (FFI) to allow the calling
language to drive the execution of its virtual machine.

Oso can read files with the `.polar` suffix, which are policy files written in Polar syntax.
These are parsed and loaded into a *knowledge base*, which can be thought of an
in-memory cache of the rules in the file.

Applications using Oso can tell it relevant information, for example registering
classes to be used with policies, which are similarly stored in the knowledge base.
The Oso implementation can now be seen as a bridge between the policy code and the application classes.

The Oso library is responsible for converting types between Oso primitive types
(like strings, numbers, and lists), and native application types (e.g. Python’s
`str`, `int`, and `list` classes), as well as keeping track of instances
of application classes.

When executing a query like `oso.query("allow", [user,
"view", expense])` Oso creates a new virtual machine to execute the query.
The virtual machine executes as a coroutine with the native library, and
therefore your application. To make authorization decisions, your application
asks Oso a question: is this (actor, action, resource) triple allowed? To answer
the question, Oso may in turn ask questions of your application: What’s the
actor’s name? What’s their organization? What’s the resource’s id? etc. The
library provides answers by inspecting application data, and control passes back
and forth until the dialog terminates with a final “yes” or a “no” answer to the
original authorization question. The virtual machine halts, and the library
returns the answer back to your application as the authorization decision.
