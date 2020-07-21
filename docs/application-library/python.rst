.. _python:

============================
Python Authorization Library
============================

Oso currently provides an authorization library to integrate oso with Python applications.

Working with Python Objects
===========================

Oso's Python authorization library allows you to write policy rules over Python objects directly.
For example:

.. code-block:: polar
    :caption: policy.polar

        allow(actor, action, resource) := actor.is_admin;

The above rule expects the ``actor`` variable to be a Python object with the attribute ``is_admin``.
The Python object is passed into Oso with a call to :py:meth:`oso.Oso.allow`:

.. code-block:: python
    :caption: app.py

        user = User()
        user.is_admin = True
        assert(oso.allow(user, "foo", "bar))

The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
called ``is_admin``, it is evaluated by the Polar rule and found to be true.

.. note::
    More detailed examples of working with application classes can be found in :ref:`auth-models`.

Supported Python Types
======================

Policy rules can be written over any Python object, but certain types of objects are supported by native Polar types.

Numbers
-------
Polar supports both integer and floating point numbers (see :ref:`basic-types`)

Strings
-------
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
-----
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
------------
Python dictionaries are mapped to Polar :ref:`dictionaries`. Likewise, dictionaries constructed in Polar
may be passed into Python methods:

.. code-block:: polar
    :caption: policy.polar
