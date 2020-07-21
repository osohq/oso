============================
Java Authorization Library
============================

Oso currently provides an authorization library to integrate oso with Java applications.

Code-level documentation is :doc:`here</java/index>`.

Working with Java Types
=======================

oso's Java authorization library allows you to write policy rules over Java objects directly.
This document explains how different types of Java objects can be used in oso policies.

.. note::
    More detailed examples of working with application classes can be found in :ref:`auth-models`.

Class Instances
^^^^^^^^^^^^^^^^
You can pass an instance of any Java class into oso and access its methods and fields from your policy.

For example:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) := actor.isAdmin;

The above rule expects the ``actor`` variable to be a Java instance with the field ``isAdmin``.
The Java instance is passed into oso with a call to ``Oso.allow``:
.. TODO: add link to javadocs

.. code-block:: java
   :caption: app.java

   public class User {
      public boolean isAdmin;

      public User(boolean isAdmin) {
         this.isAdmin = isAdmin;
      }
   }

   User user = new User(true);
   assert oso.allow(user, "foo", "bar);

The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has a field
called ``isAdmin``, it is evaluated by the Polar rule and found to be true.

Java instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Java class has been **registered** using
either the :py:meth:`~oso.Oso.register_class` function or the :py:func:`~oso.polar_class` decorator.

Registering classes also makes it possible to use :ref:`specialization` and the
:ref:`operator-matches` with the registered class:

.. code-block:: polar
   :caption: policy.polar

   allow(actor: User, action, resource) := actor matches User{name: "alice"};

.. code-block:: python
   :caption: app.py

   oso.register_class(User)

   user = User()
   user.name = "alice"
   assert(oso.allow(user, "foo", "bar))
   assert(not oso.allow("notauser", "foo", "bar"))

Numbers
^^^^^^^
Polar supports both integer and floating point numbers (see :ref:`basic-types`)

Strings
^^^^^^^
Python strings are mapped to Polar :ref:`strings`. Python's string methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) := actor.username.endswith("example.com");

.. code-block:: python
   :caption: app.py

   user = User()
   user.username = "alice@example.com"
   assert(oso.allow(user, "foo", "bar))

.. warning::
    Polar does not support methods that mutate strings in place. E.g. :py:meth:`capitalize()` will have no effect on
    a string in Polar.

Lists
^^^^^
Python lists are mapped to Polar :ref:`lists`. Python's list methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) := actor.groups.index("HR") = 0;

.. code-block:: python
   :caption: app.py

   user = User()
   user.groups = ["HR", "payroll"]
   assert(oso.allow(user, "foo", "bar"))

.. warning::
    Polar does not support methods that mutate lists in place. E.g. :py:meth:`reverse()` will have no effect on
    a list in Polar.

Likewise, lists constructed in Polar may be passed into Python methods:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) := actor.has_groups(["HR", "payroll"]);

.. code-block:: python
   :caption: app.py

   class User:
      def has_groups(groups):
            for g in groups:
               if not g in self.groups:
                  return False
            return True

   user = User()
   user.groups = ["HR", "payroll"]
   assert(oso.allow(user, "foo", "bar))

Dictionaries
^^^^^^^^^^^^
Python dictionaries are mapped to Polar :ref:`dictionaries`:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) := actor.roles = {project1: "admin", project2: "guest"};

.. code-block:: python
   :caption: app.py

   user = User()
   user.roles = {"project1": "admin", "project2": "guest"}
   assert(oso.allow(user, "foo", "bar))

Likewise, dictionaries constructed in Polar may be passed into Python methods.

