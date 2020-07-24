============================
Python Authorization Library
============================

oso is available as a :doc:`package</getting-started/download/python>` for use in Python applications.

Code-level documentation is :doc:`here<api>`.

.. toctree::
    :hidden:

    api

To install, see :doc:`installation instructions </getting-started/download/python>`.

Working with Python Objects
===========================

oso's Python authorization library allows you to write policy rules over Python objects directly.
This document explains how different types of Python objects can be used in oso policies.

.. note::
    More detailed examples of working with application classes can be found in :doc:`/using/examples/index`.

Class Instances
^^^^^^^^^^^^^^^^
You can pass an instance of any Python class into oso and access its methods and fields from your policy (see :ref:`application-types`).

Python instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Python class has been **registered** using
either the :py:meth:`~oso.Oso.register_class` method or the :py:func:`~oso.polar_class` decorator.
An example of this can be found :ref:`here <application-types>`.

Numbers and Booleans
^^^^^^^^^^^^^^^^^^^^
Polar supports both integer and floating point numbers, as well as booleans (see :ref:`basic-types`)

Strings
^^^^^^^
Python strings are mapped to Polar :ref:`strings`. Python's string methods may be accessed from policies:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.username.endswith("example.com");

.. code-block:: python
   :caption: :fab:`python` app.py

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
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.groups.index("HR") == 0;

.. code-block:: python
   :caption: :fab:`python` app.py

   user = User()
   user.groups = ["HR", "payroll"]
   assert(oso.allow(user, "foo", "bar"))

.. warning::
    Polar does not support methods that mutate lists in place. E.g. :py:meth:`reverse()` will have no effect on
    a list in Polar.

Likewise, lists constructed in Polar may be passed into Python methods:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.has_groups(["HR", "payroll"]);

.. code-block:: python
   :caption: :fab:`python` app.py

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
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.roles.project1 = "admin";

.. code-block:: python
   :caption: :fab:`python` app.py

   user = User()
   user.roles = {"project1": "admin"}
   assert(oso.allow(user, "foo", "bar))

Likewise, dictionaries constructed in Polar may be passed into Python methods.

Iterables
^^^^^^^^^
Oso handles non-list/dictionary `iterable <https://docs.python.org/3/glossary.html#term-iterable>`_ Python objects by evaluating each of the
object's elements one at a time. `Generator <https://docs.python.org/3/glossary.html#term-generator>`_ methods are a common use case for passing iterables into oso:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.get_group = "payroll";

.. code-block:: python
   :caption: :fab:`python` app.py

   class User:
      def get_group(self):
         """ Generator method to yield user groups. """
         yield from ["HR", "payroll", "]

   user = User()
   assert(oso.allow(user, "foo", "bar))

In the policy above, the right hand side of the `allow` rule will first evaluate ``"HR" = "payroll"``, then
``"payroll" = "payroll"``. Because the latter evaluation succeeds, the call to :py:meth:`~oso.Oso.allow` will succeed.
Note that if :py:meth:`get_group` returned a list, the rule would fail, as the evaluation would be ``["HR", "payroll"] = "payroll"``.

Summary
^^^^^^^

.. list-table:: Python -> Polar Types Summary
   :width: 500 px
   :header-rows: 1

   * - Python type
     - Polar type
   * - int
     - Number (Integer)
   * - float
     - Number (Float)
   * - bool
     - Boolean
   * - list
     - List
   * - dict
     - Dictionary
