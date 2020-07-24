====
RBAC
====

.. role:: polar(code)
   :language: prolog

Many authorization systems in the wild are built on a `role-based access
control <https://en.wikipedia.org/wiki/Role-based_access_control>`_ model. The
general thesis of RBAC is that the set of permissions for a system --- a
permission being the ability to perform an :ref:`action <actions>`
on a :ref:`resource <resources>` --- can be grouped into roles.

RBAC basics
-----------

Representing roles in our policy is as simple as creating :polar:`role()`
:ref:`rules <polar-rules>`:

.. todo::
   Update the below snippet once we can represent set membership.

.. literalinclude:: /examples/rbac/01-simple.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 6-20

In the above snippet of Polar, we create three :polar:`role()` rules and match
on the :polar:`actor`'s name to assign them the appropriate role. Let's write
some :polar:`allow()` rules that leverage our new roles:

.. literalinclude:: /examples/rbac/01-simple.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 21-32

To test that the roles are working, we can write a few :ref:`inline queries <inline-queries>`
in the same Polar file:

.. literalinclude:: /examples/rbac/01-simple.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 33-39

Inline queries run when the file is loaded, and check that the query after the
``?=`` succeeds.

We have a working RBAC system, but at this point it's not quite as flexible
as we'd like. For example, Deirdre is in the Accounting department, but she's
*also* an employee and should be able to submit her own expenses. We could
define a second :polar:`allow()` rule enabling accountants to :polar:`"submit"`
expenses, but it would be better to avoid that duplication and write our
policy in a way that accurately mirrors the role relationships of our
business domain. Since accountants are also employees, we can extend our
:polar:`role(actor, "employee")` rule as follows:

.. literalinclude:: /examples/rbac/02-simple.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 6-11
   :emphasize-lines: 6

Administrators should be able to do anything that accountants and employees can,
and we can grant them those permissions through the same inheritance structure:

.. literalinclude:: /examples/rbac/02-simple.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 13-19
   :emphasize-lines: 6

Now we can write a few more tests to ensure everything is hooked up correctly:

.. literalinclude:: /examples/rbac/02-simple.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 36-

RBAC with existing roles
------------------------

Our accounting firm's authorization scheme is flexible, hierarchical, and ---
let's just go ahead and say it --- beautiful. However, it's entirely based on
data that lives in our policy. One of the distinguishing features of
oso is the ability to :ref:`reach into existing domain models
<application-types>` to retrieve context for an authorization decision.


Imagine we have a ``user_roles`` database table that contains mappings
between users and the roles they've been assigned.


.. tabs::
    .. group-tab:: Python

        Our Python application has the following :py:class:`User` model that can
        look up its assigned roles from the database:

        .. literalinclude:: /examples/rbac/python/03-external.py
           :caption: :fab:`python` rbac.py
           :language: python

        By registering our application class with oso, we can begin leveraging
        it from within our policy:

        .. literalinclude:: /examples/rbac/python/04-external.py
           :caption: :fab:`python` rbac.py
           :language: python
           :emphasize-lines: 1

    .. group-tab:: Ruby

        Our Ruby application has the following :py:class:`User` model that can
        look up its assigned roles from the database:

        .. literalinclude:: /examples/rbac/ruby/03-external.rb
           :caption: :fas:`gem` rbac.rb
           :language: ruby

        By registering our application class with oso, we can begin leveraging
        it from within our policy:

        .. literalinclude:: /examples/rbac/ruby/04-external.rb
           :caption: :fas:`gem` rbac.rb
           :language: ruby
           :emphasize-lines: 7
           :start-after: user-start
           :end-before: user-end

    .. group-tab:: Java

        Our Java application has the following :py:class:`User` model that can
        look up its assigned roles from the database:

        .. literalinclude:: /examples/rbac/java/User.java
           :caption: :fab:`java` User.java
           :language: java
           :emphasize-lines: 16

        By registering our application class with oso, we can begin leveraging
        it from within our policy.

Our policy currently expects actors to be simple strings, but we can write
policy over our existing domain model by adding the :polar:`User` :ref:`type
specializer <inheritance>` to our :polar:`role()` rules:

.. literalinclude:: /examples/rbac/05-external.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 13-29

Our policy is a bit more verbose now, but don't let that distract from the
momentous shift that just occurred: by adding a single decorator to our
application model, we're now able to write rich policy over the model's
fields and methods... and we aren't finished yet!

We're still mapping users to roles in the policy despite having access to the
existing mappings through the :py:meth:`User.role` method. Let's amend that:

.. literalinclude:: /examples/rbac/06-external.polar
   :caption: :fa:`oso` rbac.polar
   :language: polar
   :lines: 1-10

There's something really powerful happening in the above that bears
highlighting: oso allowed us to not only create policies over existing
application data but, crucially, *to arrange that data in novel ways*,
enriching the pool of contextual data that informs authorization decisions
without littering complex logic all over the application. The hierarchy we
created among the :polar:`"admin"`, :polar:`"accountant"`, and
:polar:`"employee"` roles extends the existing authorization data but lives
entirely in the authorization policy and required **zero** new application code.

Summary
-------

We started with the basics of RBAC by writing out a toy policy and assigning
roles to actors in Polar. We saw how simple it is to construct arbitrary
role hierarchies, and we added a few inline queries to test our policy.

.. todo:: Below paragraph needs rephrasing.

Things started to get really interesting when we added the
:py:func:`oso.polar_class` decorator to the :py:class:`User` model, with that
one-line change to our application code unlocking the powerful pattern of
writing authorization logic directly over the fields and methods of our
existing application model.

We were able to use one of those existing methods, :py:meth:`User.role`, to
write rules over the role data stored in our application's relational
database. But we took it a step further and rearranged the existing
application roles (:polar:`"admin"`, :polar:`"accountant"`, and
:polar:`"employee"`) into a hierarchy that extended the application's
authorization system without requiring any changes to core application code.

The seasoned vets in the audience may have recognized the :polar:`actor.role`
attribute lookup for what it is: a pinch of `attribute-based access control
<https://en.wikipedia.org/wiki/Attribute-based_access_control>`_ (ABAC)
hiding amongst our RBAC policy. In the next section, we'll dive fully into
attribute-based authorization and show how intuitive it is to write concise,
flexible, and powerful ABAC rules with oso.
