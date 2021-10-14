---
title: Performance
weight: 4
aliases: 
    - ../more/performance/index.html
---

# Performance

This page explores the performance of Oso across three main axes:

**1. In practice**. How does Oso perform under typical workloads?

**2. Internals and Micro-benchmarks**. How is Oso built? What are the micro-benchmarks?

**3. Scaling**. What is the theoretical complexity of a query?

## In Practice

There are two main areas to consider when measuring the performance
of Oso queries: the time to evaluate a query relative to a policy,
and the time needed to fetch application data.

In a complex policy, the time it takes to run a single query depends on the
complexity of the *answer*. For example, a simple rule that says anyone can
“GET” the path “/” will execute in **less than 1 ms**. On the other hand,
rules that use HTTP path mapping, resource lookups, roles, inheritance, etc.
can take *approximately* **1-20 ms**. (These numbers are based on queries
executing against a local SQLite instance to isolate Oso’s performance from
the time to perform database queries.)

The time needed to fetch application data is, of course, dependent on your
specific environment and independent of Oso. Aggressive caching can be used
to reduce some of the effect of such latencies.

**Profiling**

Oso does not currently have built-in profiling tools, but this is a
high-priority item on our near-term roadmap. Our benchmark suite uses
Rust’s statistical profiling package, but is currently better suited to
optimizing the implementation than to optimizing a specific policy.

Oso has a default maximum query execution time of 30s. If you hit this maximum,
it likely means that you have created an infinite loop in your policy. You
can use the Polar debugger to help track
down such bugs.

For performance issues caused by slow database queries or too many database
queries, we recommend that you address these issues at the data access layer,
i.e., in the application. See, for example, our guidance on The “N+1 Problem”.

## Internals and Micro-benchmarks

The core of Oso is the Polar virtual machine, which is written in Rust.
(For more on the architecture and implementation, see Internals.)
A single step of the virtual machine takes approximately **1-2 us**, depending
on the instruction or *goal*. Simple operations like comparisons and assignment
typically take just a few instructions, whereas more complex operations like
pattern matching against an application type or looking up application data
need a few more. The debugger can show you the VM instructions remaining to
be executed during a query using the `goals` command.

The [current implementation](https://github.com/osohq/oso)  of Oso has
not yet been aggressively optimized for performance, but several low-hanging
opportunities for optimizations (namely, caches and indices) are on our
near-term roadmap. We do ensure that all memory allocated during a query
is reclaimed by its end, and our use of Rust ensures that the implementation
is not vulnerable to many common classes of memory errors and leaks.

You can check out our current benchmark suite in the
[repository](https://github.com/osohq/oso/tree/main/polar-core/benches),
along with instructions on how to run it. We would be delighted to accept
any example queries that you would like to see profiled; please feel free
to email us at <a href="mailto:engineering@osohq.com">engineering@osohq.com</a>.

## Scaling

At its core, answering queries against a declarative policy is a depth-first
search problem: nodes correspond to rules, and nodes are connected if a
rule references another rule in its body.

As a result, the algorithmic complexity of a policy is *in theory* very large —
exponential in the number of rules. However, *in practice* there shouldn’t be
that many distinct paths that need to be taken to make a policy decision. Oso
filters out rules that cannot be applied to the inputs early on in the
execution. What this means is that if you are hitting a scaling issue, you can
make your policies perform better by either by splitting up your rules to limit
the number of possibilities, or by adding more specializers to your rule heads.

For example, suppose you have 20 different resources, `ResourceA`, `ResourceB`,
…, and each has 10 or so `allow(actor, action, resource: ResourceA)` rules.
The performance of evaluating a rule with input of type `ResourceA` will primarily
depend on those 10 specific rules, and not the other 190 rules. In addition,
you might consider refactoring this rule to `allow(actor, action, resource:
ResourceA) if allowResourceA(actor, action, resource)`. This would mean there
are only 20 `allow` rules to sort through, and for a given resource only one
of them will ever need to be evaluated.

The performance of evaluating policies is usually independent of the number
of users or resources in the application when fetching data is handled by your
application. However, as in any programming system, you need to be on the
lookout for linear and super-linear searches. For example, if you have a method
`user.expenses()` that returns a list of the user’s expenses, the check
`expense in user.expenses()` will require O(n) VM instructions, where n
is the length of the list. It would be better to replace the linear search
with a single comparison, e.g. `expense.user_id = user.id`. Be especially
careful when nesting such rules.

## Summary

Oso typically answers simple authorization queries in **less than 1 ms**,
but may take (much) longer depending on the complexity of your rules, the
latency of application data access, and algorithmic choices. Some simple
solutions such as caching and refactoring may be used to improve performance
where needed.
