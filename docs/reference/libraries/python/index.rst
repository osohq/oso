============================
Python Authorization Library
============================

Oso currently provides an authorization library to integrate oso with Python applications.

Code-level documentation is :doc:`here<api>`.

Working with Python Objects
===========================

oso's Python authorization library allows you to write policy rules over Python objects directly.
This document explains how different types of Python objects can be used in oso policies.

.. note::
    More detailed examples of working with application classes can be found in :ref:`auth-models`.

Class Instances
^^^^^^^^^^^^^^^^
You can pass an instance of any Python class into oso and access its methods and fields from your policy.

For example:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.is_admin;

The above rule expects the ``actor`` variable to be a Python instance with the attribute ``is_admin``.
The Python instance is passed into oso with a call to :py:meth:`~oso.Oso.allow`:

.. code-block:: python
   :caption: app.py

   user = User()
   user.is_admin = True
   assert(oso.allow(user, "foo", "bar))

The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
called ``is_admin``, it is evaluated by the policy and found to be true.

Python instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Python class has been **registered** using
either the :py:meth:`~oso.Oso.register_class` method or the :py:func:`~oso.polar_class` decorator.

Registering classes also makes it possible to use :ref:`specialization` and the
:ref:`operator-matches` with the registered class:

.. code-block:: polar
   :caption: policy.polar

   allow(actor: User, action, resource) if actor matches User{name: "alice"};

.. code-block:: python
   :caption: app.py

   oso.register_class(User)

   user = User()
   user.name = "alice"
   assert(oso.allow(user, "foo", "bar))
   assert(not oso.allow("notauser", "foo", "bar"))

Once a class is registered, its class methods can also be called from oso policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor: User, action, resource) if actor.name in User.superusers();

.. code-block:: python
   :caption: app.py

   class User:
      @classmethod
      def superusers(cls):
         """ Class method to return list of superusers. """
         return ["alice", "bhavik", "clarice"]

   oso.register_class(User)

   user = User()
   user.name = "alice"
   assert(oso.allow(user, "foo", "bar))

Numbers
^^^^^^^
Polar supports both integer and floating point numbers (see :ref:`basic-types`)

Strings
^^^^^^^
Python strings are mapped to Polar :ref:`strings`. Python's string methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.username.endswith("example.com");

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
Python lists are mapped to Polar :ref:`Lists <lists>`. Python's list methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.groups.index("HR") == 0;

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

   allow(actor, action, resource) if actor.has_groups(["HR", "payroll"]);

.. code-block:: python
   :caption: app.py

   class User:
      def has_groups(self, groups):
         """ Check if a user has all of the provided groups. """
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

   allow(actor, action, resource) if actor.roles.project1 = "admin";

.. code-block:: python
   :caption: app.py

   user = User()
   user.roles = {"project1": "admin"}
   assert(oso.allow(user, "foo", "bar))

Likewise, dictionaries constructed in Polar may be passed into Python methods.

Iterables
^^^^^^^^^
Oso handles non-list/dictionary `iterable <https://docs.python.org/3/glossary.html#term-iterable>`_ Python objects by evaluating each of the
object's elements one at a time. `Generator <https://docs.python.org/3/glossary.html#term-generator>`_ methods are a common use case for passing iterables into oso:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.get_group = "payroll";

.. code-block:: python
   :caption: app.py

   class User:
      def get_group(self):
         """ Generator method to yield user groups. """
         yield from ["HR", "payroll", "]

   user = User()
   assert(oso.allow(user, "foo", "bar))

In the policy above, the right hand side of the `allow` rule will first evaluate ``"HR" = "payroll"``, then
``"payroll" = "payroll"``. Because the latter evaluation succeeds, the call to :py:meth:`~oso.Oso.allow` will succeed.
Note that if :py:meth:`get_group` returned a list, the rule would fail, as the evaluation would be ``["HR", "payroll"] = "payroll"``.
