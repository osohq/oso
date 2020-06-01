====
ABAC
====

.. role:: polar(code)
   :language: prolog

Whereas RBAC allows you to group users and permissions into pre-defined buckets,
you may also want to represent fine-grained or dynamic permissions based on *who* the user is and her relation to
the resource she wants to access. This is known as `attribute-based access
control <https://en.wikipedia.org/wiki/Attribute-based_access_control>`_ (ABAC).


ABAC basics
-----------

Continuing from the RBAC examples, suppose we want to allow employees
to view *their own* expenses. Recall that we had previously set up our
users as classes in Polar:

.. literalinclude:: /examples/rbac/04-external.py
   :language: python

We can do the same with the resources being requested:

.. literalinclude:: /examples/abac/01-simple.py
   :language: python

An ``allow()`` rule that checks that the user reading the
expense is the same person who submitted the expense, would look like:

.. literalinclude:: /examples/abac/01-simple.polar
   :language: polar
   :lines: 6-7
   :emphasize-lines: 7

This simple example shows the potential for ABAC: we took an intuitive concept
of "can see their own expenses" and represented it as a single comparison.

The power of ABAC comes from being able to express these kind of permissions
based on who you are and how you are related to the data.


ABAC ❤️ RBAC
-----------

As alluded to in the summary on RBAC, provisioning access based on checking whether
a user has a particular role is technically a simple variant of ABAC. Putting aside
whether this is a relevant distinction, the two are closely related.

The power of RBAC comes from: adding some form of organization to the limitless
distinct permissions that a person might have, and exposing those in an intuitive,
human-understandable way.

Combine this with what ABAC does best: representing relations between a user and the
data, and you get intuitive, but fine-grained permissions. For example, suppose our
company has taken off and now spans multiple locations, and now accountants can
only view expenses from their own locations. We can combine our previous roles
with some simple ABAC conditions to achieve this:

.. literalinclude:: /examples/abac/02-rbac.polar
   :language: polar
   :lines: 7-10
   :emphasize-lines: 9-10

This is great when what we need is an intersection of models, and you want to
apply both RBAC and ABAC policies simultaneously. However, the ABAC model
can be even more powerful when composed with roles. And having the roles themselves
include attributes.

For example, an employee might be an administrator of a *project*,
and therefore is allowed to see all expenses related to that project.

.. literalinclude:: /examples/abac/02-rbac.polar
   :language: polar
   :lines: 12-19
   :emphasize-lines: 18

What we can see is happening here, is that we are associated roles not just
globally to a user, but to a user for some specific resource. Other examples
might be team-, or organization- specific roles.

And these can also follow inheritance patterns like we saw with regular roles.


.. literalinclude:: /examples/abac/02-rbac.polar
   :language: polar
   :lines: 20-31


Hierachies
-----------

Up to this point, we've made a big deal about ABAC being able to represent relations
between users and resources. In the previous example, we even showed how relations
between resources permits creating inheritance logic. To expand on that idea,
here we look at representing organizational hierarchies and how these might look in
Polar.

Starting out with a simple example, suppose managers can view employees' expenses:

.. literalinclude:: /examples/abac/03-hierarchy.polar
   :language: polar
   :lines: 7-9
   :emphasize-lines: 8-9

First thing we can do, is extract out the logic for checking whether the user manages someone:

.. literalinclude:: /examples/abac/03-hierarchy.polar
   :language: polar
   :lines: 11-17

Now if we want this logic to apply for managers, and managers' managers, and so on...
then we need to make sure this logic is evaluated recursively:

.. literalinclude:: /examples/abac/03-hierarchy.polar
   :language: polar
   :lines: 18-21
   :emphasize-lines: 20-21

.. TODO: Summary
