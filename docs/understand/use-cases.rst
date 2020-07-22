Use Cases
=========

A framework for considering the spectrum of authorization use cases is as follows:
- **Customer-facing applications.** For SaaS and on-premise applications that an organization sells to its customers, how does it allow users to manage and configure permissions?
- **Internal applications.**. For SaaS and on-premise applications that an organization uses for internal employees and contractors, how does it manage and configure those permissions?
- **Infrastructure.** For infrastracture hosted in the cloud and in a company's own data centers, how does an organization manage who is allowed to do what (e.g., provision new machines, access production)?

The foundation of oso was designed to support all three of the above use cases. 

Currently, the *ideal* use case for oso is the first: customer-facing applications. The reasons for this are:
- Exposing oso as a library makes it a natural fit for adding to an application.
- Similarly, the hooks that the library provides are suited to calling into an application to act on application objects and data.
- oso *does not* handle assigning users to roles, or assigning permissions to users directly. Although you *can* do this with oso, our expectation is that this data is typically managed by the application in whatever database is already in place. oso can be used to
reference that data directly, express what roles can do in an application, and even extend the roles to include inheritance structures and hierarchies.

oso can be a good fit for internal applications where xxxxxxxxxxxxxxxxxxxxxxxxxx. Where yyyyyyyyyyyyyyyyyyyy, however, oso would not support ppppppppppppppppppppppp, and for this reason should not be seen as a replacement for something like Active Directory (at least not yet).

Regarding infrastructure: while one might be able to express their desired infrastrure policies using oso, in order to enforce those policies one would need to xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx. Currently this is possible but not documented. For this reason, oso should not be seen as a replacement for things like AWS IAM or ppppppppppppppppppppppppp.

oso has meaningful ambitions to address the full spectrum of authorization use cases for our users. In the meantime, if you have questions or particular areas of interest, we welcome feedback at engineering@osohq.com.
