.. meta::
  :description: Learn how to use oso with Python to add authorization to your application.

============================
Python Authorization Library
============================

oso is available as a :doc:`package</download>` for use in Python applications.

Code-level documentation is :doc:`here<api>`.

.. toctree::
    :hidden:

    api

To install, see :doc:`installation instructions </download>`.

**Framework integrations**

oso provides integrations for popular frameworks:

* :doc:`/using/frameworks/flask`
* :doc:`/using/frameworks/django`


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
Polar supports integer and floating point real numbers, as well as booleans (see :ref:`basic-types`).
These map to the Python ``int``, ``float``, and ``bool`` types.

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
   assert(oso.is_allowed(user, "foo", "bar))

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
   assert(oso.is_allowed(user, "foo", "bar"))

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
   assert(oso.is_allowed(user, "foo", "bar))

There is currently no syntax for random access to a list element within a policy;
i.e., there is no Polar equivalent of the Python expression ``user.groups[1]``.
To access the elements of a list, you may iterate over it with :ref:`operator-in`
or destructure it with :ref:`pattern matching <patterns-and-matching>`.

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
   assert(oso.is_allowed(user, "foo", "bar))

Likewise, dictionaries constructed in Polar may be passed into Python methods.

Iterables
^^^^^^^^^
You may iterate over any Python `iterable <https://docs.python.org/3/glossary.html#term-iterable>`_,
such as those yielded by a `generator <https://docs.python.org/3/glossary.html#term-generator>`_,
using the Polar :ref:`operator-in` operator:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if "payroll" in actor.get_groups();

.. code-block:: python
   :caption: :fab:`python` app.py

   class User:
      def get_groups(self):
         """Generator method to yield user groups."""
         yield from ["HR", "payroll"]

   user = User()
   assert(oso.is_allowed(user, "foo", "bar))

Summary
^^^^^^^

.. list-table:: Python → Polar Types Summary
   :width: 500 px
   :header-rows: 1

   * - Python type
     - Polar type
   * - int
     - Integer
   * - float
     - Float
   * - bool
     - Boolean
   * - list
     - List
   * - dict
     - Dictionary
   * - str
     - String
