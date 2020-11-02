.. meta::
  :description: Discover common authorization patterns, and how you can represent them in oso.

===============
Policy Examples
===============

In many cases, knowing *what* authorization logic you want to write
is just as hard as actually writing the logic itself. Here, we take you
through some common authorization patterns that exist, and how you
might represent and build these in oso.

.. toctree::
    :maxdepth: 1
    :hidden:

    rbac
    abac
    context
    user_types
    inheritance

:doc:`Role-based Access Control (RBAC) <rbac>`
==============================================

Role-based access control (RBAC) assigns each actor a role.  Instead of granting
permissions to individual actors, they are granted to roles.

:doc:`Read on <rbac>` to see how to implement RBAC with oso.

:doc:`Attribute-based Access Control (ABAC) <abac>`
===================================================

Attribute-based access control relies on rich attributes associated with each
actor to make authorization decisions.  This model is often used when RBAC is
not expressive enough, and is a natural extension of RBAC when using oso.

:doc:`Read on <abac>` to see how to implement ABAC with oso.

:doc:`Using Contextual Information in Authorization <context>`
==============================================================

Sometimes an authorization decision will require context beyond the
action, actor, and resource.  This could be information about the HTTP
request, or the environment the application is running in.

:doc:`Read on <context>` to see how to access contextual information within
oso policies.

:doc:`Sharing Authorization Rules Across Related Resources <inheritance>`
=========================================================================

Some applications have common authorization rules that apply to many different
types of resources.  oso policies make it possible to share rules across
related resource types, and override them as needed.

See how to use :doc:`inheritance` to implement extensible policies with oso.

:doc:`Supporting External and Internal Users <user_types>`
==========================================================

Applications may have multiple types of users.  Frequently, internal user
accounts for support reps, operations teams, or testing.  oso policies can
recognize different user types & apply different rules when necessary, avoiding
the need for multiple authorization systems.

:doc:`Continue <user_types>` to see how to write policies that distinguish
between multiple users types.
