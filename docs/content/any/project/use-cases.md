---
title: Use Cases
# Took this page down due to the Use Cases page on .com
href: https://www.osohq.com/use-cases
weight: 6
_build:
  render: never
aliases:
    - ../more/use-cases.html
---

# Use Cases

Some typical authorization use cases are:


* **Customer-facing applications.** For SaaS and on-premise
applications that an organization sells to its customers, how does
the application determine what permissions a user has?


* **Internal applications.** For
SaaS and on-premise applications that an organization uses for internal
employees and contractors,  how does
the application determine what permissions a user has?


* **User-configurable permissions.** For any application - SaaS, on-premise,
open source, etc. - where users can freely customize permissions, how does the application expose
these to users?


* **Infrastructure.** For infrastructure hosted in the cloud and in a company’s
own data centers, how does an organization manage who is allowed to do what
(e.g., provision new machines, access production)?

The foundation of Oso is designed to support all of the above use cases.

Currently, the *ideal* use case for Oso is the first: customer-facing
applications. The reasons for this are:


* Oso is currently packaged as a library, with support for various languages.
A developer can easily import the library and start using it.


* Similarly, the hooks that the library provides are designed for calling into
an application to act on application objects and data.


* Oso *does not* handle assigning users to roles, or assigning
permissions to users directly. Although you *can* do this with Oso, our
expectation is that this data is typically managed by the application in
whatever database is already in place. Oso can be used to reference that data
directly, express what roles can do in an application, and even extend the roles
to include inheritance structures and hierarchies.

Oso can be a good fit for internal applications where access might be granted on
the basis of attributes stored elsewhere, for example in Active Directory, or
GSuite. However, as above, Oso does not manage role or permission assignment directly,
and for this reason should not be seen as a
replacement for something like Active Directory (at least not yet).

We set out to build Oso to make it easier for developers to write authorization in
their applications. For those who are building frameworks or tools where developers are the target end-user, Oso might also be a good fit to give those developers fine-grained control
over permissions. We’d be happy to work together to discuss how to make that happen.

We are additionally working on exposing the same level of fine-grained control
to non-developers, which in the future would make Oso suitable for use as a
way for teams to build and expose IAM-like functionality in their products.

Regarding infrastructure: while one might be able to express her desired
infrastructure policies using Oso, in order to enforce those policies one would
need to build her own access gateway, proxy, or integration points.
Currently this is possible but not documented. For this reason,
Oso should not be seen as a replacement for
things like AWS IAM or VPN tunnels.

Oso has meaningful ambitions to address the full spectrum of authorization use
cases for users. In the meantime, if you have questions or particular areas
of interest, we welcome feedback at <a href="mailto:engineering@osohq.com">engineering@osohq.com</a>, or you can
talk to our engineering team directly through the chat widget on this page.
