=====================
Get started with oso
=====================

**oso** provides tools to authorize user actions in your application.  This
guide will gradually introduce basic concepts, with references to more detailed
documentation throughout.

This guide is written for the :doc:`Python oso library </application-library/python>`.

.. _install:

Installing oso
--------------
Download and install the ``oso`` Python wheel using ``pip install``.
The ``oso`` module requires Python version > 3.6.

Make authorization decisions
----------------------------

The primary entrypoint of ``oso`` is the :py:class:`oso.Oso` class.  This class
should be initialized in application setup, and typically will be shared
throughout:

.. testcode::

  from oso import Oso

  def setup_oso():
      oso = Oso()
      return oso

.. testoutput::
   :hide:

The :py:meth:`oso.Oso.allow` method can be used to make authorization decisions.
With oso, an authorization decision takes an **actor**, **resource** and **action**.

Add :py:meth:`oso.Oso.allow` calls anywhere in your application where an authorization needs to
be made. For example::

   from myapp.oso import get_oso
   from myapp.http import response

   def handle_read_request(request, ...):
       oso = get_oso()
       allowed = oso.allow(
           actor=request.username,
           action="read",
           resource="budget")

       if not allowed:
           return response.not_authorized()
       ...

``handle_read_request`` represents the route handler in your web framework of
choice.  Here, we are asking **oso** whether a ``read`` action for a resource
called ``budget`` is allowed.

Write policies
--------------

.. todo link below

We have not specified a policy, so this request will never be allowed.  **oso**
allows us to write requests using the **Polar language**.  Let's add a basic
Polar file to our application.

Create a file called ``policy.polar``::

  allow("alice", "read", "budget");

This simple policy contains a single **allow rule**.  It states that the actor
``"alice"`` can perform the action ``"read"`` on ``"budget"``.  Allow rules
take three parameters, the actor, action and resource.

Load this file in our setup, using :py:meth:`oso.Oso.load_file`:

.. code-block:: python
   :emphasize-lines: 5

   from oso import Oso

   def setup_oso():
       oso = Oso()
       oso.load_file("policy.polar")
       return oso


Now, if we make a request to this route with user ``"alice"`` our request will
be permitted.

Use actor properties to make authorization decisions
----------------------------------------------------

Of course, most authorization rules will be more complex than checking username
alone.

To support this, we can pass our application's user object into Polar.  Suppose
our app has a user, defined as:

.. testcode::

  import oso

  @oso.polar_class
  class User:
      def __init__(self, username: str, is_superuser: bool):
          self.username = username
          self.is_superuser = is_superuser

.. testoutput::
   :hide:

The :py:func:`oso.polar_class` function allows Polar to access the
``username`` and ``is_superuser`` fields on our application's ``User`` object.

Instead of passing the username to ``allow`` as a string, we can pass now our ``User`` object
directly:

.. code-block:: python
   :emphasize-lines: 7

   from myapp.oso import get_oso
   from myapp.http import response

   def handle_read_request(request, ...):
       oso = get_oso()
       allowed = oso.allow(
           actor=request.user,
           action="read",
           resource="budget")

       if not allowed:
           return response.not_authorized()
       ...

Now, our allow rule can check for the superuser attribute::


  allow(actor, "read", "budget") :=
      actor.is_superuser = true;

In this rule, we have used a body, indicated by the ``:=`` operator. ``user``
defines a variable, which is bound to the value of ``actor``. In a rule with a body,
the portion of the rule before the ``:=`` operator (called the **head**) must first match.
Then, the ``body`` portition is evaluated.

This rule will allow any **actor** that is a superuser to ``read`` the ``budget`` resource.


We aren't just limited to accessing attributes from Polar.  Suppose our ``User``
object has been extended to load a user's role from our database:

.. code-block:: python
  :emphasize-lines: 9,10

  import oso

  @oso.polar_class
  class User:
      def __init__(self, username: str, is_superuser: bool):
          self.username = username
          self.is_superuser = is_superuser

      def role(self):
          return db.users.get_role(self)

We can add a new authorization rule using this method::

  allow(actor, "write", "budget") :=
      actor.role() = "admin";

This rule states that actors whose role method returns ``admin`` can write to ``budget``.

What's next
===========

In this guide, we've covered how to install oso, and write basic Polar rules over our
application's domain models.

To continue, either:

1. Explore :doc:`RBAC </auth-models/rbac>` or :doc:`ABAC </auth-models/abac>` authorization models.
2. Learn more about :doc:`authorization fundementals </auth-fundamentals>` with oso.
3. Dive deeper into the :doc:`Polar language </language/index>`.
