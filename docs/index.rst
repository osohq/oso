.. oso documentation master file, created by
   sphinx-quickstart on Fri Mar 20 10:34:51 2020.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.


Welcome to the oso documentation!
==================================


.. admonition:: What is oso?

    oso is a library for adding authorization to applications using a declarative
    policy language

The core use case of oso is to add authorization logic to any application.
This is commonly solved by custom logic sprinkled ad-hoc throughout an application,
leading to code that is hard to maintain, modify, and debug.

oso is built on the following principles:

- **Separation of concerns.** Authorization logic is distinct from business logic. By separating the two, you can make changes to the policy which apply across the entire application, write reusable patterns, and get a single place to control, test and visualize access.
- **Right tool for the right job.** Authorization deals in facts and rules about who is allowed to do what in a system. Solutions to describe authorization logic ought to be declarative and have semantics that map to common domain concepts – like roles and relationships, or whether a policy is satisfied given certain inputs.
- **Authorization decisions and application data are inseparable.** Authorization decisions always rely on application context – like who a user is and her relationship to the data she's trying to access. The authorization system ought to be able to call directly into the application so you can write policies using applications objects and data directly.

To see these principles in action, :doc:`continue on to the Getting Started guide <getting-started>`.

To learn more about oso and how these principles motivated its design, 
:doc:`read the oso overview page <oso-overview>`.
