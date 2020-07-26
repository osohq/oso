========
Security
========

This page is split into two sections with two distinct purposes: 
1. Security best practices for using oso, and
2. Our approach to building a secure product

-----------------------
Security Best Practices
-----------------------

Policy Authoring
----------------

To reduce the likelihood of writing logic bugs in oso policies, we
recommend using :ref:`support for specializers as type checks <specializer>`
wherever possible.

For policy errors that are most likely due to incorrect policies, such as
accessing attributes that don't exist, oso returns hard errors.

Some problems are reported as warnings, when the problem *may* be a logic
bug. An example of this is :ref:`singletons (unused variables) <singletons>`.

We additionally recommend the use of :ref:`inline-queries` as simple policy unit
tests. Since oso is accessible as a library, you should authorization as
part of your application test suite.

Change Management
-----------------

As a reminder, oso typically replaces authorization logic that would
otherwise exist in your application. By using oso, you are able to
move much of that logic into separate a policy file/s, which is easier to
audit and watch for changes.

Currently, the best practice for policy change management is to treat oso
like regular source code. You should use code review practices and CI/CD
to make sure you have properly vetted and kept a history (e.g., through git) of all changes to authorization logic.

Auditing
--------

If you are interested in capturing an audit log of *policy decisions*,
and being able to understand *why* oso authorized a request, please
`contact us <https://osohq.com/company/contact-us>`_.

-----------------------------------------
Our Approach to Building a Secure Product
-----------------------------------------

Code
----

The core of oso is written in Rust, which vastly reduces the risk of memory
unsafety relative to many other low-level and embeddable languages (e.g., C, C++). The oso engineering team codes defensively – we makes extensive use of types, validate inputs,
and handle errors safely.

All source code is available at `our GitHub repository <https://github.com/osohq/oso>`_.
Releases are built and published using GitHub actions.

The current version of oso is in Developer Preview and has not yet undergone a
code audit. We plan to engage a qualified third-party to perform an audit, whose results we will make publicly available, in the near future.


Vulnerability Reporting
-----------------------

We appreciate any efforts to find and disclose vulnerabilities to us.

If you would like to report an issue, or have any other security questions or concerns, please email us at :email:`security@osohq.com`.
