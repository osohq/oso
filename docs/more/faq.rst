===
FAQ
===

--------------------------------------------------------------------------
How is oso different from authentication and other authorization products?
--------------------------------------------------------------------------

First, oso handles *authorization only* and does not handle authentication, and
as a result is not comparable to products that handle things like user
management, password reset, multi-factor authentication, etc.

As for how oso differs from other authorization solutions -- such as LDAP,
authorization libraries (e.g,. Pundit, Casbin), or end-to-end IAM solutions like
Okta -- there are 3 main ways:

**Integration with the application.**

oso integrates directly in your application as a library. In this way, it is
more akin to Pundit, in that you write policies with access to application
objects, and can make authorization decisions wherever is suitable. Furthermore,
getting started with oso means adding the oso package to your applications
requirements and you are good to go.

Although oso gives you the experience of using a native library, the core of oso
is actually written is Rust, but designed in such a way that it can be embedded
as a library in any language. If you application is written in multiple languages,
different teams can all use oso for a consistent approach.

With standalone services like Okta, you can do a limited amount of
authorization, such as basic roles, and anything more complex requires
synchronizing data with the service.

**How you define rules.**

oso policies are written using the declarative language Polar.
Many other authorization system which allow you to
add users to groups/roles, and maybe associate those with
particular methods/routes. Whereas oso polices can be as expressive
as you need.

There have been other examples of policy languages, like XACML and Rego.
Or other declarative approaches like AWS IAM, and Hashicorp's Sentinel.

The key difference between oso and these languages is that oso allows you to
write policies directly over your application data, access object attributes,
call class methods, and use things like inheritance already represented in your
application.

In addition, one of the biggest challenges with using a new policy language or
domain-specific language (DSL) is the need to learn an entirely new syntax. When
designing the Polar language, we *(1)* kept the syntax as simple as possible,
and *(2)* allow you to leverage objects and *methods* that exist in your
language. This means you don't have another API to learn, and aren't limited by
what our language supports.

**????**

-----------------------------------
How do I integrate oso into my app?
-----------------------------------

There are two main steps to adding oso.

First of all, you express authorization logic as declarative rules, written in Polar
syntax and stored in oso policy files.

Second, you install the oso library for your application language/framework,
and add the ``is_allowed`` checks to wherever it is most suitable for your use case.
For example, it is common to have checks at the API layer -- for example checking
the HTTP request, and the path supplied -- as well as checks on the data access,
e.g. when your application is retrieving data from the database.

For more detailed discussion on where to integrated oso in your application
depending on your requirements, please visit :doc:`/getting-started/application/index`.

-------------------------
What data does oso store?
-------------------------

When you load policy files into oso, oso stores in memory the rules defined in
the policy. In addition, any registered classes are stored on the oso instance.

In the course of executing a query, oso will cache any instances of classes/objects
that it sees, but these are cleared when the query finishes.

oso *does not*, for example, store any data about the users, what groups they
are in, or what permissions they have been assigned. The expectation is that
this data lives :doc:`in your application <design-principles>` and is
accessed by oso when evaluating queries.

Because of this, it is rare to need to change policies while the application
is running. For example, if a user's access needs to be revoked, they leave
a company, or they change role, then by updating the application data this
change will be reflected immediately in policy decisions.

Changes to policy should be seen as the same as making source code changes,
and can be implemented through existing deployment processes.

----------------------------
Can I query oso arbitrarily?
----------------------------

Absolutely, you can!

We use ``allow`` as convention to make it easy to get started with oso.
However, all oso libraries additionally expose a ``query_predicate`` method,
that allows you to query any rule you want.

Beyond this, you can even query using inputs which are not yet set, by
passing in variables. However, this is currently experimental, and
full documentation is coming soon.

----------------------------------------
How does oso access my application data?
----------------------------------------

When a policy contains an attribute or method lookup, e.g. ``actor.email``, the
policy evaluation pauses and control is returned to the host.
An event is created to say "please lookup the field ``email`` on the object
``instance with id 123``" (the oso library stores a lookup from instance IDs to the
concrete application instance).

What happens next depends on the specific language. But it will use some form of
dynamic lookup -- maybe a simple ``getattr`` in Python, or reflection in Java.

The result is returned to the policy engine, and execution continues.

-------------------------------------------------------------------------------------------------
What is the best practice for managing policy files in a way that's maintainable in the long-run? 
-------------------------------------------------------------------------------------------------

This is a common question from those who have used policy languages or rules
engines before. Corollary questions may be:

- Can I have multiple policy files?
- How do I stop policy files from getting out of control?


The answer, of course, varies by use case, but we suggest the following rules of thumb:

- Yes, you can and should have multiple policy files. All rules loaded
  into oso live in the same namespace. So you can reference rules in other
  policy files without importing.
- We encourage you to think of your policy files the same way you think
  about source code. Large rules should be refactored into smaller
  rules, where each rule captures a self-contained piece of logic.
- Source files can be organized according to which components they refer to.

------------------------------------------------
What are the performance characteristics of oso?
------------------------------------------------

oso is designed to be lightweight and have a limited performance footprint
within your application. The core library is written in Rust, and is
driven by your application. There are no background threads, no GC, no
IO to wait on. Each instruction takes about 1-2us, and typical queries
will take around 1-20ms.

For a more detailed discussion of the performance characteristics of oso,
please the :doc:`performance page <performance/index>`.

----------------------------------------------------
Use cases, i.e., When to use and when not to use oso
----------------------------------------------------

The foundation of oso is designed to support a wide variety of use cases, though
given oso's focus on application integration there are some use cases that are
currently more natural fit than others. For a more detailed discussion of this
topic, please see our :doc:`use cases page <use-cases>`.

-------
Pricing
-------

oso is freely available as an open source product.
For support pricing, please `contact us <https://osohq.com/company/contact-us>`_.

-------
License
-------

oso licensed under the `Apache 2.0 license <https://github.com/osohq/oso/blob/master/LICENSE>`.

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
