---
title: Security
weight: 5
aliases: 
    - ../more/security.html
---

# Security

This page is split into two sections with two distinct purposes:
1. Security best practices for using Oso, and
2. Our approach to building a secure product

## Security Best Practices

### Policy Authoring

To reduce the likelihood of writing logic bugs in Oso policies, we
recommend using support for specializers as type checks
wherever possible.

For policy errors that are most likely due to incorrect policies, such as
accessing attributes that don’t exist, Oso returns hard errors.

Some problems are reported as warnings, when the problem *may* be a logic
bug. An example of this is singletons (unused variables).

We additionally recommend the use of Inline Queries (?=) as simple policy unit
tests. Since Oso is accessible as a library, you should test authorization as
part of your application test suite.

### Change Management

As a reminder, Oso typically replaces authorization logic that would
otherwise exist in your application. By using Oso, you are able to
move much of that logic into separate a policy file/s, which is easier to
audit and watch for changes.

Currently, the best practice for policy change management is to treat Oso
like regular source code. You should use code review practices and CI/CD
to make sure you have properly vetted and kept a history (e.g., through git) of all changes to authorization logic.

### Auditing

If you are interested in capturing an audit log of *policy decisions*,
and being able to understand *why* Oso authorized a request, please
[contact us](https://osohq.com/company/contact-us).

## Our Approach to Building a Secure Product

### Code

The core of Oso is written in Rust, which vastly reduces the risk of memory
unsafety relative to many other low-level and embeddable languages (e.g., C, C++). The Oso engineering team codes defensively – we make extensive use of types, validate inputs,
and handle errors safely.

All source code is available at [our GitHub repository](https://github.com/osohq/oso).
Releases are built and published using GitHub actions.

Oso has not yet undergone a code audit. We plan to engage a qualified third-party to perform an audit,
whose results we will make publicly available, in the near future.

### Vulnerability Reporting

We appreciate any efforts to find and disclose vulnerabilities to us.

If you would like to report an issue, or have any other security questions or concerns, please email us at <a href="mailto:security@osohq.com">security@osohq.com</a>.
