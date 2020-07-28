===
FAQ
===

-----------------------------------
How do I integrate oso into my app?
-----------------------------------

There are two main steps to adding oso.

First, you express authorization logic as declarative rules, written in Polar and stored in oso policy files.

Second, you install the oso library for your application language/framework,
and add the ``is_allowed`` checks to wherever it is most suitable for your use case.
For example, it is common to have checks at the API layer -- for example checking
the HTTP request, and the path supplied -- as well as checks on the data access,
e.g. when your application is retrieving data from the database.

For more detailed discussion on where to integrate oso in your application
depending on your requirements, please visit our guide, :doc:`/getting-started/application/index`.

-------------------------
What data does oso store?
-------------------------

When you load policy files into oso, oso stores in memory the rules defined in
the policy. In addition, oso stores any registered classes on the oso instance.

In the course of executing a query, oso caches any instances of classes/objects
that it sees, but it clears these when the query finishes.

oso *does not*, for example, store any data about the users, what groups they
are in, or what permissions have been assigned to them. The expectation is that
this data lives :doc:`in your application <design-principles>`, and that oso accesses it as needed when evaluating queries.

Because of this, it is rare to need to change policies while the application
is running. For example, if you need to revoke a user's access because they leave
the company or change roles, then updating the application data will immediately flow through to policy decisions and achieve the desired outcome.

Changes to policy should be seen as the same as making source code changes,
and can be implemented through existing deployment processes.

----------------------------
Can I query oso arbitrarily?
----------------------------

Absolutely, you can!

We use ``allow`` as convention to make it easy to get started with oso.
However, all oso libraries additionally expose a ``query_rule`` method,
which enables you to query any rule you want.

Beyond this, you can even query using inputs that are not yet set by
passing in variables. However, this is currently an experimental feature, and
full documentation is coming soon.

----------------------------------------
How does oso access my application data?
----------------------------------------

When a policy contains an attribute or method lookup, e.g., ``actor.email``, the
policy evaluation pauses and oso returns control to the application.
It creates an event to say "please lookup the field ``email`` on the object
``instance with id 123``". (The oso library stores a lookup from instance IDs to the
concrete application instance.)

What happens next depends on the specific language, but it will use some form of
dynamic lookup -- e.g., a simple ``getattr`` in Python, or reflection in Java.

The application returns the result to the policy engine, and execution continues.

-------------------------------------------------------------------------------------------------
What is the best practice for managing policy files in a way that's maintainable in the long-run? 
-------------------------------------------------------------------------------------------------

This is a common question from those who have used policy languages or rules
engines before. Corollary questions may be:

- Can I have multiple policy files?
- How do I stop policy files from getting out of control?


The answer, of course, varies by use case, but we suggest the following rules of thumb:

- Yes, you can and should have multiple policy files. All rules loaded
  into oso live in the same namespace; you can reference rules in other
  policy files without importing.
- We encourage you to think of your policy files the same way you think
  about source code. You should refactor large rules into smaller
  rules, where each rule captures a self-contained piece of logic.
- You can organize source files according to the components they refer to.

------------------------------------------------
What are the performance characteristics of oso?
------------------------------------------------

oso is designed to be lightweight and to have a limited performance footprint. The core library is written in Rust, and is
driven directly by your application. There are no background threads, no garbage collection, no
IO to wait on. Each instruction takes about 1-2 us, and typical queries take approximately 1-20 ms.

For a more detailed discussion of the performance characteristics of oso,
please the :doc:`performance page <performance/index>`.

-----------------------------------------------------------------------------
Use cases, i.e., When should I use oso, and when should I use something else?
-----------------------------------------------------------------------------

The foundation of oso is designed to support a wide variety of use cases, though
given oso's focus on application integration there are some use cases that are
currently a more natural fit than others. For a more detailed discussion of this
topic, please see our :doc:`use cases page <use-cases>`.

-------
Pricing
-------

oso is freely available as an open source product.
For support pricing, please `contact us <https://osohq.com/company/contact-us>`_.

-------
License
-------

oso licensed under the *`Apache 2.0 license <https://github.com/osohq/oso/blob/master/LICENSE>`_*.

---------------------------------------------
What languages and frameworks do you support?
---------------------------------------------

We currently support Python, Ruby and Java, and are actively working on supporting more languages.
We are also in the process of writing documentation for native framework support.

Vote & track your favorite language and framework integrations at our 
`Github repository <https://github.com/osohq/oso>`_.

--------------------------------------
What operating systems do you support?
--------------------------------------

We currently support Linux and Mac OS X.
We have initial Windows support, and expect publish a release for Windows soon.

Sign up for our Newsletter if you'd like to stay up to speed on the latest product updates.
