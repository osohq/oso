===========
Performance
===========

This page explores the performance of oso across three main axes
to help you understand the expected performance of oso:

- **In practice**. How does oso perform under typical workloads?
- **Microbenchmarks**. How is oso built? What are the microbenchmarks?
- **Scaling**. What is the theoretical complexity of a query?
  How do the performance numbers scale across orders of magnitude?


In Practice
-----------

There are two main areas of performance to consider when measuring the
performance of oso queries: the time for policy evaluation, and the time to
fetch application data.

Within a complex policy, the time it takes to run a single query depends on the
complexity of the *answer*. For example, if we have a simple rule that says
anyone can "GET" the path "/", this will execute in **less than 1ms**.

On the other hand, if the rule being queried makes use of: HTTP path mapping,
resource lookups, roles, inheritance, etc. this takes *approximately*
**1-20ms**.

These numbers are based on queries executing with a local sqlite instance, so
performance is approximately independent of the time to perform database
queries.

**Profiling**

Currently oso does not have any built in profiling tools, but these are a high
priority on our roadmap.

There is a maximum execution time set to 30s. If you hit this, it likely means
that either you have many expensive lookups, or have created an infinite
loop/recusive call in your policy.

For any performance issues caused by slow or too many database queries, our
recommendation is to solve these issues at the data access layer -- in the
application.

See, for example, our guidance on :doc:`n_plus_one`.

Benchmarks
----------

The core oso is Polar. This is written in Rust, and creates a virtual machine to
execute queries (for more on this, see :doc:`/more/internals`).

A single step of the virtual machine takes approximately **1-2us** (depending on
the instruction).

Simple operations like comparisons and assignment will typically need just a
few instruction, whereas more complex operations like using specializers
to check if the input matches an application type, or looking up application
data will need a few more.

The `current implementation <https://github.com/osohq/oso>`_  of oso has not
been aggressively performance optimized. It uses Rust's built in
reference-counting to clean up any values creating in the execution of a query.

You can see our benchmarking suite in the
`repository <https://github.com/osohq/oso/tree/main/polar/benches>`_,
along with instructions on how to run them.

We would be delighted to accept any example queries that people would like to
see profiled.

Scaling
-------

At its core, answering queries against a declarative policy is a depth-first
search problem. Where nodes correspond to rules, and nodes are connected if a
rule references another rule :ref:`in its body <combining_rules>`.

However, our implementation has a few algorithmic crucial optimizations: rule
filtering and sorting.

Rule filtering does a quick pass through all of the target rules to filter out
any rules which do no match in one of the input arguments. This means, for
example, that if you have many, many ``allow`` rules for different actions, or
different resources, that this initial pass will be filtered down to just those
rules.

Rule sorting is a crucial part of the language semantics: like method resolution
order, we evaluate rules in order of most-to-least specific. When combined with
the filtering above, this can often be a performance win as well: since a more
specific rule is more likely to contain the desired logic.


.. toctree::
    :hidden:

    Overview <self>
    n_plus_one
