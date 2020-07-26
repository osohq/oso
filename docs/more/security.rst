========
Security
========


Code
----

The core of oso is written in Rust and vastly reduces the risk of memory
unsafety. We code defensively, making extensive use of types, validating inputs,
and handling errors safely.

All source code is available at `our GitHub repository <https://github.com/osohq/oso>`_.
Releases are built and published using GitHub actions.

The current version of oso is in developer preview and has not yet undergone a
code audit, but we plan to do so soon.


Vulnerability Reporting
-----------------------

We appreciate any efforts to find and disclose vulnerabilities to us.

If you would like to report an issue, or have any other security concerns
you would like to discuss, please email us at :email:`security@osohq.com`.


Policy Authoring
----------------

In order to reduce the likelihood of writing logic bugs in oso policies, we
recommend using :ref:`support for specializers as type checks <specializer>`
wherever possible.

For policy errors that are most likely due to incorrect policies, such as
accessing attributes that don't exist, we return hard errors.

Some problems are reported as warnings, when the problem *may* be a logic
bug, such as is the case with :ref:`singletons (unused variables) <singletons>`.

We additionally recommend the use of :ref:`inline-queries` as simple policy unit
tests. Since oso is accessible as a library, authorization should be tested as
part of the application test suite.

Change Management
-----------------

As a reminder, oso is typically replacing authorization logic that would
have already existed in your application. By using oso, you are able to
move much of that logic into separate policy files, which is easier to
audit and watch for changes.

Currently, the best practice for policy change management is treating oso
like regular source code. You should use code review practices, and CI/CD
to make sure all changes to authorization logic has been vetted.

Auditing
--------

If you are interested in capturing an audit log of policy decisions,
and being able to audit *why* a decision was made, please
`contact us <https://osohq.com/company/contact-us>`_.
