============================
Python Authorization Library
============================

Oso currently provides an authorization library to integrate oso with Python applications.

Installing the oso Python library
==================================

Download and install the oso Python wheel and install using ``pip install oso``.
The ``oso`` module requires Python version > 3.6.

Using the oso library
=====================

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

Polar policy files are loaded using :py:meth:`oso.Oso.load_file`, or strings
can be loaded directly with `oso.Oso.load_str`.
Once a policy is loaded, the :py:meth:`oso.Oso.allow` method can be used to make
:ref:`authorization queries <queries>`.

Add ``oso.allow()`` calls anywhere in your application where an authorization needs to
be made.

Registering application classes with Polar
==========================================

Python classes can be registered as :ref:`application-types` with Polar using either the :py:func:`oso.register_class` function or the
:py:func:`oso.polar_class` decorator.

The class's fields, class variables, and methods may then be accessed from within Polar as well.

For example:

.. literalinclude:: /externals-example/company.py
   :start-after: company-start
   :end-before: company-end

In this example, the ``name`` and ``member`` methods define Polar-exposed attributes
that are accessible within Polar.

``department_members`` is an example of an exposed method that takes an argument.

Polar-exposed methods may be generator functions, since any variable in Polar can take many values.
They may yield multiple times, and the Polar engine will attempt evaluation with each yield
result. Methods can also return a single value.

.. _testing:

Testing a Polar policy
===================================

Testing policies with the oso library is easy using `pytest <https://docs.pytest.org/en/latest/>`_.

Let's write a quick policy using our ``Company`` class:

.. literalinclude:: /externals-example/company.polar
   :language: polar
   :start-after: policy-start
   :end-before: policy-end

If we save the above rules to the file ``company.polar``, the following test should pass:

.. literalinclude:: /externals-example/test_company.py
   :start-after: test-company-start
   :end-before: test-company-end
