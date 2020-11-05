.. meta::
  :description: Learn about the internals of the oso policy engine and Polar language, which are underpinned by a VM built in Rust.

Internals
---------

oso is supported in :doc:`a number of languages </using/libraries/index>`,
but the `oso core <https://github.com/osohq/oso>`_ is written in Rust,
with bindings for each specific language.

At the core of oso is the **Polar language**. This handles parsing
policy files and executing queries in the form of a virtual machine. oso was
designed from the outset to be natively embedded in different
languages. It exposes a foreign function interface (FFI) to allow the calling
language to drive the execution of its virtual machine.

oso can read files with the ``.polar`` suffix, which are policy files written in Polar syntax.
These are parsed and loaded into a *knowledge base*, which can be thought of an
in-memory cache of the rules in the file.

Applications using oso can tell it relevant information, for example registering
classes to be used with policies, which are similarly stored in the knowledge base.
The oso implementation can now be seen as a bridge between the policy code and the application classes.

The oso library is responsible for converting types between oso primitive types
(like strings, numbers, and lists), and native application types (e.g. Python's
``str``, ``int``, and ``list`` classes), as well as keeping track of instances
of application classes.

When executing a query like ``oso.query("allow", [user,
"view", expense])`` oso creates a new virtual machine to execute the query.
The virtual machine executes as a coroutine with the native library, and
therefore your application. To make authorization decisions, your application
asks oso a question: is this (actor, action, resource) triple allowed? To answer
the question, oso may in turn ask questions of your application: What's the
actor's name? What's their organization? What's the resource's id? etc. The
library provides answers by inspecting application data, and control passes back
and forth until the dialog terminates with a final "yes" or a "no" answer to the
original authorization question. The virtual machine halts, and the library
returns the answer back to your application as the authorization decision.
