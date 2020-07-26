===========
Performance
===========

This page explores the performance of oso across three main axes
to help you understand the topic:

- **In practice**. How does oso perform under typical workloads?
- **Internals and Microbenchmarks**. How is oso built? What are the micro-benchmarks?
- **Scaling**. What is the theoretical complexity of a query?


In Practice
-----------

There are two main areas consider when measuring the
performance of oso queries: the time to evaluate a policy, and the time to
fetch application data.

Within a complex policy, the time it takes to run a single query depends on the
complexity of the *answer*. For example, if we have a simple rule that says
anyone can "GET" the path "/", this will execute in **less than 1 ms**. On the other hand, if the rule you are querying makes use of: HTTP path mapping,
resource lookups, roles, inheritance, etc. this can take *approximately*
**1-20 ms**. These numbers are based on queries executing with a local sqlite instance to isolate oso's performance as best as possible from the time to perform database
queries.

The time to fetch application data is, of course, dependent on your specific environment and independent of oso.

**Profiling**

oso does not currently have built-in profiling tools, but this is a high-priority item
priority on our near-term roadmap.

oso has a default maximum execution time set to 30s. If you hit this maximum, it likely means
that either you have many expensive lookups, or have created an infinite
loop / recursive call in your policy.

For any performance issues caused by slow database queries or too many database queries, we recommend that you address these issues at the data access layer, i.e., in the
application. See, for example, our guidance on :doc:`n_plus_one`.

Internals and Micro-benchmarks
------------------------------

The core of oso is Polar. This is written in Rust, and creates a virtual machine to
execute queries (for more on this, see :doc:`/more/internals`).

A single step of the virtual machine takes approximately **1-2 us** (depending on
the instruction).

Simple operations like comparisons and assignment typically need just a
few instructions, whereas more complex operations like using specializers
to check if the input matches an application type, or looking up application
data need a few more.

The `current implementation <https://github.com/osohq/oso>`_  of oso has not yet
been aggressively performance optimized. It uses Rust's built-in
reference-counting to clean up any values created in the execution of a query.

You can see our benchmark suite in the
`repository <https://github.com/osohq/oso/tree/main/polar/benches>`_,
along with instructions on how to run them.

We would be delighted to accept any example queries that people would like to
see profiled. Please feel free to email us at :email:`engineering@osohq.com`.

Scaling
-------

At its core, answering queries against a declarative policy is a depth-first
search problem. Where nodes correspond to rules, and nodes are connected if a
rule references another rule :ref:`in its body <combining_rules>`.

As a result, the algorithmic complexity of a policy is *in theory* very large —
exponential in the number of rules. However, *in practice* there shouldn't be
that many distinct paths that need to be taken to make a policy decision. oso
filters out rules that cannot be applied to the inputs early on in the
execution. What this means is that if you are hitting a scaling issue you can
make your policies perform better either by splitting up rules to limit the
number of possibilities, or by adding more qualifiers to a rule such.

For example, if you have 20 different resources, ``ResourceA``, ``ResourceB``, ...,
and each has 10 or so ``allow(actor, action, resource: ResourceA)`` rules. The
performance of evaluating a rule with input of type `ResourceA` will primarily
depend on those 10 related rules, and not the other 190 rules. In addition, you
might consider refactoring this rule to ``allow(actor, action, resource:
ResourceA) if allowResourceA(actor, action, resource)`` . This means there are
only 20 ``allow`` rules to sort through, and for a given resource only one of
these will even need to be evaluated.

The performance of evaluating policies is usually independent of the number of
users or resources in the application when fetching data is handled by your
application. However, if a large amount of data is returned to oso for making a
policy decision, it's potentially very costly.

For example, if you have a method ``user.expenses()`` that returns a list of the
user's expenses, and you want to check ``expense in user.expenses()``, this will
be an ``O(n)`` operation, in terms of Polar VM instructions. It would be better to
avoid this by reworking this, e.g. ``expense.user_id = user.id``.


.. toctree::
    :hidden:

    Overview <self>
    n_plus_one
